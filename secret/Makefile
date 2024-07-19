build-release:
	@echo "Building release..."
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

build-debug:
	@echo "Building debug..."
	cargo build --target wasm32-unknown-unknown

build: build-release

optimize:
	@echo "Optimizing..."
	./optimize.sh

release: build optimize

clean:
	@echo "Cleaning..."
	cargo clean
	rm -rf artifacts/


generate-schema:
	cd bridge && cargo schema
	cd collection-deployer && cargo schema
	cd snip721 && cargo schema
	cd snip1155 && cargo schema
	cd storage721 && cargo schema
	cd storage1155 && cargo schema
	cd storage-deployer && cargo schema

generate-ts: generate-schema
	cd bridge && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/bridge --name Bridge --no-bundle
	cd snip721 && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/snip721 --name Snip721 --no-bundle
	cd snip1155 && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/snip1155 --name Snip1155 --no-bundle
	cd collection-deployer && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/collection-deployer --name CollectionDeployer --no-bundle
	cd storage721 && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/storage721 --name Storage721 --no-bundle
	cd storage1155 && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/storage1155 --name Storage1155 --no-bundle
	cd storage-deployer && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/storage-deployer --name StorageDeployer --no-bundle
