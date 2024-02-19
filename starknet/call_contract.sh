#!/usr/bin/env bash

# Prompt the user to enter the calldata
calldata=$(<calldata)

# Pass the calldata to the sncast command
sncast --profile testnet \
  --wait \
  invoke \
  --contract-address 0x3ba45c52dfa67d8c85f75001706f9fd5e34ab582b87d7f536f347ce35584ffc \
  --function "verify_and_register_fact" \
  --calldata $calldata
