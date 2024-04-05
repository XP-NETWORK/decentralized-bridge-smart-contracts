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
	cd collection_deployer && cargo schema
	cd cosm_nft && cargo schema
	cd nft_store && cargo schema
	cd store_deployer && cargo schema

generate-ts: generate-schema
	cd bridge && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/bridge --name Bridge --no-bundle
	cd cosm_nft && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/cosm_nft --name CosmosNft --no-bundle
	cd nft_store && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/nft_store --name NftStore --no-bundle
	cd collection_deployer && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/collection_deployer --name CollectionDeployer --no-bundle
	cd store_deployer && cosmwasm-ts-codegen generate --plugin client --schema ./schema --out ../clients/src/store_deployer --name StorageDeployer --no-bundle
