# Zerosync-StarkNet Proof Verification Toolkit

This toolkit facilitates the verification and validation of programs using the Zerosync protocol. Below are the setup instructions and usage guidelines to seamlessly integrate Zerosync proofs into your workflow.

## Setup

### Install Cairo Lang

Execute the following commands to install the Cairo language and its dependencies:

```bash
cd cairo-lang
pip install --upgrade pip
zip -r cairo-lang-0.12.0.zip cairo-lang-0.12.0
pip install cairo-lang-0.12.0.zip
pip install aiofiles colorama
cd ../
```

## Usage

### Compile `increment_batch.cairo` file

Compile your Cairo file `increment_batch.cairo` into JSON format:

```bash
cairo-compile increment_batch.cairo --output increment_batch.json --no_debug_info --proof_mode
```

### Confirm Valid Program

Ensure your program is valid by verifying its hash:

```bash
python hash_program.py increment_batch.json
```

The output should match the expected hash: `0x1ff70c9838765d61370402a62551f9c00518efbfa098f882b285f0db646943b` as specified in the Zerosync demo [here](https://zerosync.org/demo/).

### Confirm Valid Program Bootloader Hash

Verify the bootloader hash of your program:

```bash
python hash_program_bootloader.py --program increment_batch.json
```

The output should be: `0x34400f14ac9c420fb903fcf409cebaf1adca6c2ac4405c743d480fb6a07b9e2`.

### Download Zerosync Proof Pair

Retrieve the Zerosync proof pair for validation:

```bash
curl https://zerosync.org/demo/proofs/latest/air-public-input.json > air-public-input.json
curl https://zerosync.org/demo/proofs/latest/aggregated_proof.bin > aggregated_proof.bin
```

### Parse Sandstone Binary Proof to Cairo Compatible Format

Convert the Sandstone binary proof to a Cairo-compatible format:

```bash
cargo +nightly run --release --manifest-path header_chain_parser/Cargo.toml aggregated_proof.bin air-public-input.json increment_batch.json increment_batch_proof.json proof
```

### Compile Incrementer and Bootloader

Compile the incrementer and bootloader:

```bash
python setup.py
```

### Verify Proof Locally with Bootloaded Incrementer and Prove Its Execution

Verify the proof locally with the bootloaded incrementer and prove its execution:

```bash
python bootloader_increment_batch.py
```

### Verify proof on StarkNet

Check out the proof:

```bash
cd starknet
cargo run --release < ../stone-prover/e2e_test/bootloader_proof.json > calldata
./call.sh
cd ../
```