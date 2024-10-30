echo "building..."

cd bridge

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..

cd collection-deployer

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..

cd snip721

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..


cd snip1155

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..

cd storage-deployer

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..

cd storage721

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..

cd storage1155

RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

cd ..