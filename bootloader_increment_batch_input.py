import json
from hash_program import *

def run():
    # Define input variables
    compiled_program = "increment_batch.json"
    program_input = "increment_batch_input.json"
    output_file = "cairo-lang/simple_bootloader_input.json"

    # Read compiled_program
    with open(compiled_program, 'r') as f:
        compiled_program_data = json.load(f)
    
    # Read program_input
    with open(program_input, 'r') as f:
        program_input_data = json.load(f)

    task = {}

    task['type'] = "RunProgramTask"
    task['program'] = compiled_program_data
    task['program_input'] = program_input_data
    task['use_poseidon'] = False

    data = {
        "tasks": [task],
        "single_page": True
    }

    # Write output
    with open(output_file, 'w') as f:
        json.dump(data, f)

if __name__ == "__main__":
    run()