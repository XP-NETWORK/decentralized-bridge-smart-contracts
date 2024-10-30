#!/bin/sh


echo "Creating Artifacts directory..."
mkdir -p artifacts/


for WASM in target/wasm32-unknown-unknown/release/*.wasm; do
    OUT_FILENAME=$(basename "$WASM" .wasm).wasm
    echo "Optimizing $OUT_FILENAME ..."
    /home/d/multiversx-sdk/wasm-opt/version_112/binaryen-version_112/bin/wasm-opt -Os --signext-lowering "$WASM" -o "artifacts/$OUT_FILENAME"
    cat "artifacts/$OUT_FILENAME" | gzip -9 > ./artifacts/$OUT_FILENAME.gz
done

echo "Post-processing artifacts..."
(
  cd artifacts
  # create hashes
  sha256sum -- *.wasm | tee checksums.txt
)