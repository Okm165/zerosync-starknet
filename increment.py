import subprocess
from colorama import Fore, Style

def log_and_run(commands, description, cwd=None):
    full_command = " && ".join(commands)
    try:
        print(f"{Fore.YELLOW}Starting: {description}...{Style.RESET_ALL}")
        print(f"{Fore.CYAN}Command: {full_command}{Style.RESET_ALL}")
        result = subprocess.run(full_command, shell=True, check=True, cwd=cwd, text=True)
        print(f"{Fore.GREEN}Success: {description} completed!\n{Style.RESET_ALL}")
    except subprocess.CalledProcessError as e:
        print(f"{Fore.RED}Error running command '{full_command}': {e}\n{Style.RESET_ALL}")

log_and_run([
    'prev_proof="increment_batch_proof.json"',
    'increment_input_template="increment_input_template.json"',
    'outputFile="increment_batch_input.json"',
    'jq -n \
    --argfile prev_proof "$prev_proof" \
    -f "$increment_input_template" > "$outputFile"'
], "Preapring increment_batch input", cwd=".")

log_and_run([
    "cairo-run \
    --program=increment_batch.json \
    --layout=recursive \
    --program_input=increment_batch_input.json \
    --air_public_input=increment_batch_public_input.json \
    --air_private_input=increment_batch_private_input.json \
    --trace_file=increment_batch_trace.bin \
    --memory_file=increment_batch_memory.bin \
    --print_output \
    --proof_mode \
    --print_info"
], "Running increment_batch", cwd=".")