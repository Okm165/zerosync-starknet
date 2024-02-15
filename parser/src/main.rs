use ark_serialize::CanonicalDeserialize;
use clap::Parser;
use clap::Subcommand;
use ministark::stark::Stark;
use ministark::Proof;
use ministark_gpu::fields::p3618502788666131213697322783095070105623107215331596699973092056135872020481;
use p3618502788666131213697322783095070105623107215331596699973092056135872020481::ark::Fp;
use sandstorm::claims::recursive::CairoVerifierClaim;
use sandstorm_binary::AirPublicInput;
use sandstorm_binary::CompiledProgram;
use sandstorm_parser::memory::Writeable;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command(name = "parser")]
#[command(about = "A parser for reencoding STARK proofs", long_about = None)]
struct Cli {
    path: String,        //PROOF
    path_inputs: String, //AIR_PUBLIC_INPUT
    path_program: String,
    path_output: String, // "parser/test/output.json"
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // StarkConfig,
    // PublicInput,
    // StarkUnsentCommitment,
    // StarkWitness

    // // StarkConfig
    // // TracesConfig,
    // TableCommitmentConfig,
    // FriConfig,
    // ProofOfWorkConfig,
    Proof,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let proof_file = File::open(cli.path)?;
    let program_inputs_file = File::open(cli.path_inputs)?;
    let program_file = File::open(cli.path_program)?;
    let path_output = cli.path_output;

    let air_public_input: AirPublicInput<Fp> =
        serde_json::from_reader(program_inputs_file).unwrap();
    let program: CompiledProgram<Fp> = serde_json::from_reader(program_file).unwrap();
    let claim = CairoVerifierClaim::new(program, air_public_input.clone());

    // Load the proof and its public inputs from file
    let proof: Proof<CairoVerifierClaim> = Proof::deserialize_compressed(proof_file).unwrap();
    println!("proof_options {:?}", proof.options);
    let metadata = claim.verify(proof.clone(), 80).unwrap();

    // Serialize to Cairo-compatible memory
    let json_arr = match &cli.command {
        Commands::Proof => (metadata.query_positions, proof, air_public_input).to_cairo_memory(),
        // Commands::PublicInputs => proof.public_inputs.to_cairo_memory(),
    };

    let mut f = File::create(path_output).unwrap();
    write!(f, "{}", json_arr)
}
