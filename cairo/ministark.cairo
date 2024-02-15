
%builtins range_check

from starkware.cairo.common.alloc import alloc

from starkware.cairo.common.bool import FALSE
from starkware.cairo.common.math import assert_le
from starkware.cairo.common.math_cmp import is_le

struct Vec {
    n_elements: felt,
    elements: felt*,
}

struct CubicFelt {
    c0: felt,
    c1: felt,
    c2: felt,
}

struct CubicVec {
    n_elements: felt,
    elements: CubicFelt*,
}

struct Digest {
    element_0: felt,
    element_1: felt,
    element_2: felt,
    element_3: felt,
    element_4: felt,
    element_5: felt,
    element_6: felt,
    element_7: felt,
}

struct ProofOptions {
    num_queries: felt,
    lde_blowup_factor: felt,
    grinding_factor: felt,
    fri_folding_factor: felt,
    fri_max_remainder_size: felt,
}

struct MerkleProofs {
    n_proofs: felt,
    n_digests: felt, // number of digests per proof
    digests: Digest*,
}

struct FriProofLayer {
    values: Vec,
    proofs: MerkleProofs,
    commitment: Digest,
}

struct FriProof {
    layers: FriProofLayer*,
    remainder: Vec,
    remainder_commitment: Digest,
}

struct Queries {
    base_trace_values: Vec, // Fp
    extension_trace_values: CubicVec, // Fq
    composition_trace_values: CubicVec, // Fq
    base_trace_proofs: MerkleProofs,
    extension_trace_proofs: MerkleProofs,
    composition_trace_proofs: MerkleProofs,
}

struct PublicMemory {
    length: felt,
    memory: felt*, // (usize + 1, Fp)
}

// ExecutionInfo
struct PublicInputs {
    initial_ap: felt,
    initial_pc: felt,
    final_ap: felt,
    final_pc: felt,
    range_check_min: felt,
    range_check_max: felt,
    public_memory: PublicMemory,
    public_memory_padding_address: felt,
    public_memory_padding_value: felt,
}

struct Proof {
    options: ProofOptions,
    trace_len: felt,
    base_trace_commitment: Digest,
    extension_trace_commitment: Digest, // optional
    composition_trace_commitment: Digest,
    fri_proof: FriProof,
    pow_nonce: felt,
    trace_queries: Queries,
    public_inputs: PublicInputs,
    execution_trace_ood_evals: CubicVec, // Fq
    composition_trace_ood_evals: CubicVec, // Fq
}

func num_fri_layers{range_check_ptr}(proof_options: ProofOptions, domain_size) -> felt {
    assert_le(proof_options.fri_folding_factor, proof_options.fri_max_remainder_size);
    return __num_fri_layers(domain_size, proof_options.fri_folding_factor, proof_options.fri_max_remainder_size, 0);
}

func __num_fri_layers{range_check_ptr}(domain_size, folding_factor, max_remainder_size, num_layers) -> felt {
    let is_domain_size_too_small = is_le(domain_size, max_remainder_size);
    if (is_domain_size_too_small != FALSE) {
        return num_layers;
    }
    assert_le(domain_size / folding_factor, domain_size - 1);
    return __num_fri_layers(domain_size / folding_factor, folding_factor, max_remainder_size, num_layers + 1);
}

func remainder_size{range_check_ptr}(proof_options: ProofOptions, domain_size) -> felt {
    assert_le(proof_options.fri_folding_factor, proof_options.fri_max_remainder_size);
    let is_domain_size_too_small = is_le(domain_size, proof_options.fri_max_remainder_size);
    if (is_domain_size_too_small != FALSE) {
        return domain_size;
    }
    assert_le(domain_size / proof_options.fri_folding_factor, domain_size - 1);
    return remainder_size(proof_options, domain_size / proof_options.fri_folding_factor);
}
