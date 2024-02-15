# Usage

```bash
source ~/cairo_venv/bin/activate


# generate proof
cairo-run --program_input ./parser/test/bootloader_inputs.json \
    --program ./parser/test/bootloader_compiled.json \
    --air_private_input ./parser/test/air-private-input.json \
    --air_public_input ./parser/test/air-public-input.json \
    --trace_file ./parser/test/trace.bin \
    --memory_file ./parser/test/memory.bin \
    --layout recursive \
    --min_steps 128 \
    --proof_mode \
    --print_info
(cd ../sandstorm-mirror && cargo +nightly build -r -F parallel,asm) && ../sandstorm-mirror/target/release/sandstorm --program ./parser/test/bootloader_compiled.json \
    --air-public-input ./parser/test/air-public-input.json \
    prove --air-private-input ./parser/test/air-private-input.json \
          --output ./parser/test/bootloader-proof.bin

# generate recursive proof
cairo-compile cairo/test_recurse.cairo --proof_mode --output cairo/test_recurse.json
cairo-run --layout recursive \
    --program cairo/test_recurse.json \
    --trace_file cairo/trace.bin \
    --memory_file cairo/memory.bin \
    --min_steps 128 \
    --proof_mode \
    --print_info
(cd ../sandstorm-mirror && cargo +nightly build -r -F parallel,asm) && ../sandstorm-mirror/target/release/sandstorm --program ./cairo/test_recurse.json \
    --air-public-input ./cairo/air-public-input.json \
    prove --air-private-input ./cairo/air-private-input.json \
          --output ./recursive-proof.bin
```



# Test cairo verifier

```bash
source ~/cairo_venv/bin/activate
cd parser
cargo +nightly run
cd ..
cairo-compile cairo/test_recurse.cairo --proof_mode --output cairo/test_recurse.json
cairo-run --layout starknet --program cairo/test_recurse.json --trace_file cairo/trace.bin --memory_file cairo/memory.bin --min_steps 128 --proof_mode
```

# Generate proof

```bash
cairo-run --program_input ./parser/test/bootloader_inputs.json --program ./parser/test/bootloader_compiled.json --air_private_input ./parser/test/air-private-input.json --air_public_input ./parser/test/air-public-input.json --trace_file ./parser/test/trace.bin --memory_file ./parser/test/memory.bin --layout recursive --min_steps 128 --proof_mode --print_info

(cd ../sandstorm-mirror && cargo +nightly build -r -F parallel,asm) && ../sandstorm-mirror/target/release/sandstorm --program ./parser/test/bootloader_compiled.json \
    --air-public-input ./parser/test/air-public-input.json \
    prove --air-private-input ./parser/test/air-private-input.json \
          --output ./parser/test/bootloader-proof.bin


cairo-run --layout recursive --program cairo/test_recurse.json --air_private_input ./cairo/air-private-input.json --air_public_input ./cairo/air-public-input.json --trace_file cairo/trace.bin --memory_file cairo/memory.bin --min_steps 128 --proof_mode  --print_info
```






```
cairo-compile cairo/test_recurse.cairo \
    --proof_mode \
    --output cairo/test_recurse.json \
    --cairo_path ../cairo-lang-parser/src

cairo-run --layout starknet 
    --program cairo/test_recurse.json 
    --trace_file cairo/trace.bin 
    --memory_file cairo/memory.bin 
    --min_steps 128 
    --proof_mode
```