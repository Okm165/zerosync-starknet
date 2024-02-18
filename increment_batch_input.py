import json
from hash_program import *

def run():
    # Define input variables
    compiled_program = "increment_batch.json"
    prev_proof_file = "increment_batch_proof.json"
    output_file = "increment_batch_input.json"

    data = {}

    # Read prev_proof.json
    with open(prev_proof_file, 'r') as f:
        prev_proof_data = json.load(f)

    # Calculate program hash
    with open(compiled_program, 'r') as file:
        program = fetch_compiled_program(file)
        program_hash = compute_hash_chain(program.data)

    data['increment_program_hash'] = program_hash
    data['prev_proof'] = prev_proof_data
    data['batch_size'] = 8

    # Write output
    with open(output_file, 'w') as f:
        json.dump(data, f)

if __name__ == "__main__":
    run()