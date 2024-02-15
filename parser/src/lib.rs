use blake2::Blake2s256;
use ministark::hash::Digest;
use sandstorm::claims::recursive::CairoVerifierClaim;
use sandstorm_crypto::hash::pedersen::PedersenDigest;
use sandstorm_crypto::hash::pedersen::PedersenHashFn;
use sandstorm_crypto::merkle::mixed::MixedMerkleDigest;
use sandstorm_crypto::merkle::FriendlyMerkleTreeProof;
use std::collections::BTreeSet;
use std::iter::zip;
pub mod memory;
use sandstorm::claims::NUM_FRIENDLY_COMMITMENT_LAYERS;
use sandstorm_binary::AirPublicInput;
use sandstorm_binary::Layout;
use sandstorm_layouts::recursive::AirConfig;
use memory::DynamicMemory;
use memory::Writeable;
use ministark::air::AirConfig as _;
use ministark::fri::fold_positions;
use ministark::utils::SerdeOutput;
use ministark::Proof;
use ministark_gpu::fields::p3618502788666131213697322783095070105623107215331596699973092056135872020481::ark::Fp;
use num_bigint::BigUint;
use ruint::aliases::U256;

impl Writeable for BigUint {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_hex_value(format!("{:#01x}", self))
    }
}

impl Writeable for U256 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_hex_value(format!("{:#01x}", self))
    }
}

impl Writeable for u128 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_hex_value(format!("{:#01x}", self))
    }
}

impl Writeable for Fp {
    fn write_into(&self, target: &mut DynamicMemory) {
        BigUint::from(*self).write_into(target)
    }
}

impl<D: digest::Digest> Writeable for SerdeOutput<D> {
    fn write_into(&self, target: &mut DynamicMemory) {
        let num = U256::try_from_be_slice(self).unwrap();
        let element = Fp::from(BigUint::from(num));
        element.write_into(target);
    }
}

impl Writeable for MixedMerkleDigest<PedersenDigest, SerdeOutput<Blake2s256>> {
    fn write_into(&self, target: &mut DynamicMemory) {
        match self {
            MixedMerkleDigest::HighLevel(d) => d.write_into(target),
            MixedMerkleDigest::LowLevel(d) => d.write_into(target),
        }
    }
}

// StarkProof
impl Writeable for (Vec<usize>, Proof<CairoVerifierClaim>, AirPublicInput<Fp>) {
    fn write_into(&self, target: &mut DynamicMemory) {
        let (query_positions, proof, public_input) = self;

        let fri_options = proof.options.into_fri_options();
        let lde_domain_size = proof.options.lde_blowup_factor as usize * proof.trace_len;

        // StarkConfig
        let mut stark_config = target.alloc();

        // TracesConfig
        let mut traces = stark_config.alloc();
        // original: TableCommitmentConfig
        let mut original = traces.alloc();
        // n_columns
        AirConfig::NUM_BASE_COLUMNS.write_into(&mut original);
        // VectorCommitmentConfig
        let mut vector = original.alloc();
        // height
        lde_domain_size.ilog2().write_into(&mut vector);
        // NOTE: number of pedersen hashes in the merkle path
        // n_verifier_friendly_commitment_layers
        NUM_FRIENDLY_COMMITMENT_LAYERS.write_into(&mut vector);
        // interaction: TableCommitmentConfig
        let mut interaction = traces.alloc(); // TableCommitmentConfig
        AirConfig::NUM_EXTENSION_COLUMNS.write_into(&mut interaction); // n_columns
        let mut vector = interaction.alloc(); // VectorCommitmentConfig
        lde_domain_size.ilog2().write_into(&mut vector); // height
        NUM_FRIENDLY_COMMITMENT_LAYERS.write_into(&mut vector); // n_verifier_friendly_commitment_layers

        // composition: TableCommitmentConfig
        let mut composition = stark_config.alloc();
        // n_columns
        // NOTE: constraint evaluation blowup factor
        2u32.write_into(&mut composition);
        let mut vector = composition.alloc(); // VectorCommitmentConfig
        lde_domain_size.ilog2().write_into(&mut vector); // height
        NUM_FRIENDLY_COMMITMENT_LAYERS.write_into(&mut vector); // n_verifier_friendly_commitment_layers

        // FriConfig
        let fri_options = proof.options.into_fri_options();
        let log_fri_folding_factor = proof.options.fri_folding_factor.ilog2();
        let mut fri_config = stark_config.alloc();
        let num_layers = fri_options.num_layers(lde_domain_size);
        // log_input_size
        lde_domain_size.ilog2().write_into(&mut fri_config);
        // num merkle layers + 1 remainder layer
        (num_layers + 1).write_into(&mut fri_config); // n_layers
        let mut inner_layers = fri_config.alloc(); // TableCommitmentConfig
        for k in 0..num_layers {
            proof
                .options
                .fri_folding_factor
                .write_into(&mut inner_layers); // n_columns
            let mut vector = inner_layers.alloc(); // VectorCommitmentConfig
            let height = lde_domain_size.ilog2() - log_fri_folding_factor * (k + 1) as u32;
            height.write_into(&mut vector); // height
            NUM_FRIENDLY_COMMITMENT_LAYERS.write_into(&mut vector); // n_verifier_friendly_commitment_layers
        }
        let fri_step_sizes = [
            vec![0u32],
            vec![log_fri_folding_factor; fri_options.num_layers(lde_domain_size)],
        ]
        .concat();
        fri_config.write_array(&fri_step_sizes); // fri_step_sizes
        (fri_options.remainder_size(lde_domain_size) / proof.options.lde_blowup_factor as usize)
            .ilog2()
            .write_into(&mut fri_config); // TODO: log_last_layer_degree_bound

        // ProofOfWorkConfig
        let proof_of_work_bits = proof.options.grinding_factor;
        let mut proof_of_work = stark_config.alloc();
        proof_of_work_bits.write_into(&mut proof_of_work); // n_bits

        proof.trace_len.ilog2().write_into(&mut stark_config); // log_trace_domain_size
        proof.options.num_queries.write_into(&mut stark_config); // n_queries
        proof
            .options
            .lde_blowup_factor
            .ilog2()
            .write_into(&mut stark_config); // log_n_cosets
        NUM_FRIENDLY_COMMITMENT_LAYERS.write_into(&mut stark_config); // n_verifier_friendly_commitment_layers

        // PublicInput
        let mut public_input = target.alloc();
        self.2.n_steps.ilog2().write_into(&mut public_input); // log_n_steps
        self.2.rc_min.write_into(&mut public_input); // rc_min
        self.2.rc_max.write_into(&mut public_input); // rc_max
        self.2.layout.sharp_code().write_into(&mut public_input); // layout
        public_input.write_array(&[0u32]); // TODO: dynamic_params

        let segments = {
            let mut res = vec![
                self.2.memory_segments.program,
                self.2.memory_segments.execution,
                self.2.memory_segments.output.unwrap(),
                self.2.memory_segments.pedersen.unwrap(),
                self.2.memory_segments.range_check.unwrap(),
            ];

            match self.2.layout {
                Layout::Recursive => res.push(self.2.memory_segments.bitwise.unwrap()),
                Layout::Starknet => unimplemented!(),
                _ => unimplemented!(),
            }

            res
        };

        segments.len().write_into(&mut public_input);
        let mut segment_infos = public_input.alloc(); // SegmentInfo
        for segment in segments {
            segment.begin_addr.write_into(&mut segment_infos);
            segment.stop_ptr.write_into(&mut segment_infos);
        }

        let public_memory_padding = self.2.public_memory_padding();
        public_memory_padding.address.write_into(&mut public_input);
        public_memory_padding.value.write_into(&mut public_input);

        self.2.public_memory.len().write_into(&mut public_input); // main_page_len
        let mut main_page = public_input.alloc(); // AddrValue
        for memory_entry in &self.2.public_memory {
            memory_entry.address.write_into(&mut main_page); // address
            memory_entry.value.write_into(&mut main_page); // value
        }

        // don't use any continious pages at the moment
        // only use main page
        0u32.write_into(&mut public_input); // n_continuous_pages
        0u32.write_into(&mut public_input); // continuous_page_headers

        let base_trace_commitment = proof.base_trace_commitment.clone();
        let extension_trace_commitment = proof.extension_trace_commitment.clone().unwrap();
        let composition_trace_commitment = proof.composition_trace_commitment.clone();

        // StarkUnsentCommitment
        let mut stark_unsent_commitment = target.alloc();
        // traces: TracesUnsentCommitment
        let mut traces = stark_unsent_commitment.alloc();
        // original.vector.commitment_hash.value: ChannelUnsentFelt
        base_trace_commitment.write_into(&mut traces);
        // TableUnsentCommitment::VectorUnsentCommitment::ChannelUnsentFelt
        // TODO: interaction.vector.commitment_hash.value:
        extension_trace_commitment.write_into(&mut traces);
        // TODO: composition.vector.commitment_hash.value:
        composition_trace_commitment.write_into(&mut stark_unsent_commitment);
        let mut oods_values = stark_unsent_commitment.alloc();
        for eval in &proof.execution_trace_ood_evals {
            // proof_elements.push(U256::from(BigUint::from(eval)));
            // proof_elements.push(U256::from(to_montgomery(*eval)));
            eval.write_into(&mut oods_values);
        }
        for eval in &proof.composition_trace_ood_evals {
            println!("ood eval: {}", eval);
            eval.write_into(&mut oods_values);
        }
        // fri: FriUnsentCommitment
        let mut fri = stark_unsent_commitment.alloc();
        // - inner_layers: TableUnsentCommitment*
        let mut inner_layers = fri.alloc();
        for layer in &proof.fri_proof.layers {
            layer.commitment.write_into(&mut inner_layers);
        }
        // last_layer_coefficients: ChannelUnsentFelt*
        let mut last_layer_coefficients = fri.alloc();
        for coeff in &proof.fri_proof.remainder_coeffs {
            coeff.write_into(&mut last_layer_coefficients);
        }
        let mut proof_of_work = stark_unsent_commitment.alloc(); // TODO: ProofOfWorkUnsentCommitment
        proof.pow_nonce.write_into(&mut proof_of_work); // TODO nonce.value: ChannelUnsentFelt

        // StarkWitness
        let mut stark_witness = target.alloc();
        // traces_decommitment: TracesDecommitment

        {
            let mut traces_decommitment = stark_witness.alloc();
            // NOTE: base trace
            // original: TableDecommitment*
            let base_trace_vals = &proof.trace_queries.base_trace_values;
            let mut original = traces_decommitment.alloc();
            base_trace_vals.len().write_into(&mut original); // TODO: n_values
            let mut original_values = original.alloc(); // values: felt*
            for val in &proof.trace_queries.base_trace_values {
                val.write_into(&mut original_values)
            }

            // NOTE: extension trace
            // interaction: TableDecommitment*
            let extension_trace_vals = &proof.trace_queries.extension_trace_values;
            let mut interaction = traces_decommitment.alloc(); // TableDecommitment
            extension_trace_vals.len().write_into(&mut interaction); // TODO: n_values
            let mut interaction_values = interaction.alloc(); // values: felt*
            for val in &proof.trace_queries.extension_trace_values {
                val.write_into(&mut interaction_values)
            }
        }

        {
            let base_trace_merkle_witness = get_merkle_witness_values(
                &proof.base_trace_commitment,
                proof.trace_queries.base_trace_proof.clone(),
                &query_positions,
            );
            // traces_witness: TracesWitness*,
            let mut traces_witness = stark_witness.alloc();
            let mut original = traces_witness.alloc(); // TableCommitmentWitness
            let mut vector = original.alloc(); // VectorCommitmentWitness
            base_trace_merkle_witness.len().write_into(&mut vector); // TODO: n_authentications
            let mut original_authentications = vector.alloc();
            for v in base_trace_merkle_witness {
                v.write_into(&mut original_authentications);
            }

            // let extension_trace_merkle_proofs =
            //     partition_proofs(&proof.trace_queries.extension_trace_proofs);
            let extension_trace_merkle_witness = get_merkle_witness_values(
                proof.extension_trace_commitment.as_ref().unwrap(),
                proof.trace_queries.extension_trace_proof.clone().unwrap(),
                &query_positions,
            );
            // TableCommitmentWitness
            let mut interaction = traces_witness.alloc();
            let mut vector = interaction.alloc(); // VectorCommitmentWitness
            extension_trace_merkle_witness.len().write_into(&mut vector); // TODO: n_authentications
            let mut interaction_authentications = vector.alloc();
            for v in extension_trace_merkle_witness {
                v.write_into(&mut interaction_authentications);
            }
        }

        {
            // composition_decommitment: TableDecommitment
            let composition_trace_vals = &proof.trace_queries.composition_trace_values;
            let mut composition_decommitment = stark_witness.alloc();
            composition_trace_vals
                .len()
                .write_into(&mut composition_decommitment);
            let mut composition_values = composition_decommitment.alloc(); // values: felt*
            for val in &proof.trace_queries.composition_trace_values {
                val.write_into(&mut composition_values)
            }

            // let composition_trace_merkle_proofs =
            //     partition_proofs(&proof.trace_queries.composition_trace_proofs);
            let composition_trace_merkle_witness = get_merkle_witness_values(
                &proof.composition_trace_commitment,
                proof.trace_queries.composition_trace_proof.clone(),
                query_positions,
            );
            // TableCommitmentWitness
            let mut composition_witness = stark_witness.alloc();
            let mut vector = composition_witness.alloc();
            composition_trace_merkle_witness
                .len()
                .write_into(&mut vector); // TODO: n_authentications
            let mut composition_authentications = vector.alloc();
            for v in composition_trace_merkle_witness {
                v.write_into(&mut composition_authentications);
            }
        }

        // FriWitness
        let mut fri_witness = stark_witness.alloc();
        let mut fri_witness_layer = fri_witness.alloc();
        let mut indices = query_positions.clone();
        for layer in &proof.fri_proof.layers {
            // let mut fri_witness_layer = fri_witness_layers.alloc();
            let folding_factor = proof.options.fri_folding_factor as usize;
            println!("about to wold with folding_factor={folding_factor}");
            let folded_indices = fold_positions(&indices, folding_factor);
            println!("folded_indices={folded_indices:?}");

            // let pos_set = BTreeSet::from_iter(indices);
            // for (pos, coset) in zip(fold_positions(&indices, folding_factor), cosets) {
            //     for (i, v) in coset.iter().enumerate() {
            //         if !posSet.contains(&(pos * N + i)) {
            //             fri_proof.push(U256::from(to_montgomery(*v)))
            //         }
            //     }
            // }

            let cosets = layer
                .flattenend_rows
                .chunks(proof.options.fri_folding_factor as usize);
            let pos_set = BTreeSet::from_iter(&indices);
            let mut leaves = Vec::new();
            for (pos, coset) in zip(&folded_indices, cosets) {
                for (i, v) in coset.iter().enumerate() {
                    if !pos_set.contains(&(pos * folding_factor + i)) {
                        // leaves.push(U256::from(to_montgomery(*v)))
                        leaves.push(*v)
                    }
                }
            }

            leaves.len().write_into(&mut fri_witness_layer); // n_leaves
            let mut fri_leaves = fri_witness_layer.alloc();
            println!("sibling[0] {}", leaves[0]);

            for leaf in leaves {
                leaf.write_into(&mut fri_leaves);
            }

            /// FRI proof should always be multi column commitment
            let batched_proofs = match &layer.merkle_proof {
                FriendlyMerkleTreeProof::MultiCol(p) => p.clone(),
                FriendlyMerkleTreeProof::SingleCol(_) => unreachable!(),
            };
            let num_authentications =
                batched_proofs.sibling_leaves.len() + batched_proofs.nodes.len();
            // table_witness: TableCommitmentWitness
            let mut table_witness = fri_witness_layer.alloc();
            // vector: VectorCommitmentWitness
            let mut vector_witness = table_witness.alloc();
            num_authentications.write_into(&mut vector_witness);
            let mut fri_authentications = vector_witness.alloc();
            for authentication in batched_proofs.sibling_leaves {
                authentication.write_into(&mut fri_authentications);
            }
            for authentication in batched_proofs.nodes {
                authentication.write_into(&mut fri_authentications);
            }

            // match partition_proofs(&layer.proofs) {
            //     MerkleProofsVariant::Hashed(ref proofs) => {}
            //     MerkleProofsVariant::Unhashed(ref proofs) => {
            //         let batched_proofs = BatchedMerkleProof::from_proofs(proofs,
            // &folded_indices);         let num_authentications =
            //             batched_proofs.sibling_leaves.len() + batched_proofs.nodes.len();
            //         // table_witness: TableCommitmentWitness
            //         let mut table_witness = fri_witness_layer.alloc();
            //         // vector: VectorCommitmentWitness
            //         let mut vector_witness = table_witness.alloc();
            //         num_authentications.write_into(&mut vector_witness);
            //         let mut fri_authentications = vector_witness.alloc();
            //         for authentication in batched_proofs.sibling_leaves {
            //             authentication.write_into(&mut fri_authentications);
            //         }
            //         for authentication in batched_proofs.nodes {
            //             authentication.write_into(&mut fri_authentications);
            //         }
            //     }
            // }

            indices = folded_indices;
        }
    }
}

fn get_merkle_witness_values(
    root: &MixedMerkleDigest<PedersenDigest, SerdeOutput<Blake2s256>>,
    proof: FriendlyMerkleTreeProof<PedersenHashFn>,
    indices: &[usize],
) -> Vec<U256> {
    match proof {
        FriendlyMerkleTreeProof::MultiCol(proof) => {
            let mut leaf_siblings = Vec::new();
            let mut initial_leaves = Vec::new();
            for leaf in proof.initial_leaves {
                initial_leaves.push(U256::from_be_bytes::<32>(leaf.as_bytes()))
            }
            for sibling in proof.sibling_leaves {
                leaf_siblings.push(U256::from_be_bytes::<32>(sibling.as_bytes()))
            }

            let nodes = proof
                .nodes
                .into_iter()
                .map(|digest| U256::from_be_bytes::<32>(digest.as_bytes()))
                .collect::<Vec<U256>>();

            let shift = 1 << proof.height;
            let adjusted_indices = indices.iter().map(|i| i + shift);

            [leaf_siblings, nodes].concat()
        }
        FriendlyMerkleTreeProof::SingleCol(_) => todo!("handle single column case"),
    }
}
