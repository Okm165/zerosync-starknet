%builtins pedersen range_check bitwise

// from starkware.cairo.cairo_verifier.objects import CairoVerifierOutput
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, HashBuiltin
from starkware.cairo.stark_verifier.air.layouts.starknet.verify import verify_proof
from starkware.cairo.stark_verifier.core.stark import StarkProof

// source ~/cairo_venv/bin/activate
// cairo-compile cairo/test_recurse.cairo --proof_mode --output cairo/test_recurse.json
// cairo-run --layout starknet --program cairo/test_recurse.json --trace_file cairo/trace.bin --memory_file cairo/memory.bin --min_steps 128 --proof_mode

const SECURITY_BITS = 18;

// Main function for the Cairo verifier.
//
// Hint arguments:
// program_input - Contains the inputs for the Cairo verifier.
//
// Outputs the program hash and the hash of the output.
func main{pedersen_ptr: HashBuiltin*, range_check_ptr, bitwise_ptr: BitwiseBuiltin*}() {
    alloc_locals;

    let proof_mem: StarkProof* = alloc();
    local proof: StarkProof* = proof_mem;
    %{
        import json
        import subprocess

        # ministark_parser = subprocess.run(["parser/target/debug/sandstorm_parser", "example/test_parser.proof", 'proof'], capture_output=True)
        # ministark = ministark_parser.stdout
        # ministark_json = json.loads(ministark)
        ministark_file = open("parser/output.json")
        ministark_json = json.load(ministark_file)
        ministark_file.close()

        # Note the following:
        # - Addresses are stored as `Relocatable` values in the Cairo VM.
        # - The "+" operator is overloaded to perform pointer arithmetics.
        # - Felts are hex encoded starting with "0x". The virtual addresses are encoded as decimals.
        segments.write_arg(ids.proof.address_, [(int(x, 16) if x.startswith('0x') else ids.proof.address_ + int(x)) for x in ministark_json])
    %}

    verify_proof(proof=proof, security_bits=SECURITY_BITS);

    return ();
}