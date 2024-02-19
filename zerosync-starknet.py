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
    "curl https://zerosync.org/demo/proofs/latest/air-public-input.json > air-public-input.json"
], "Download Zerosync Proof Pair", cwd=".")

log_and_run([
    "curl https://zerosync.org/demo/proofs/latest/aggregated_proof.bin > aggregated_proof.bin"
], "Download Zerosync Proof Pair", cwd=".")

log_and_run([
    "cargo +nightly run --release --manifest-path header_chain_parser/Cargo.toml aggregated_proof.bin air-public-input.json increment_batch.json increment_batch_proof.json proof"
], "Parse Sandstone Binary Proof to Cairo Compatible Format", cwd=".")

log_and_run([
    "python bootloader_increment_batch.py"
], "Verify Proof Locally with Bootloaded Incrementer and Prove Its Execution", cwd=".")

log_and_run([
    "cargo run --release < ../stone-prover/e2e_test/bootloader_proof.json > calldata && ./call_contract.sh"
], "Verify proof on StarkNet", cwd="starknet")