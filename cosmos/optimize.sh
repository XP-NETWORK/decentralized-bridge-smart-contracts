#!/bin/sh


echo "Creating Artifacts directory..."
mkdir -p artifacts/

for WASM in target/wasm32-unknown-unknown/release/*.wasm; do
    OUT_FILENAME=$(basename "$WASM" .wasm).wasm
    echo "Optimizing $OUT_FILENAME ..."
    wasm-opt -Os --signext-lowering "$WASM" -o "artifacts/$OUT_FILENAME"
done

echo "Post-processing artifacts..."
(
  cd artifacts
  # create hashes
  sha256sum -- *.wasm | tee checksums.txt
)