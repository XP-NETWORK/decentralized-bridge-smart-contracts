build-nft-storage: build-tokens
	@echo "Building nft-storage"
	ligo compile contract src/NFTStorage.jsligo -m NFTStorage -o build/NFTStorage.tz
	ligo compile contract src/NFTStorage.jsligo -m NFTStorage -o build/NFTStorage.json  --michelson-format json 

build-sft-storage: build-tokens
	@echo "Building sft-storage"
	ligo compile contract src/SFTStorage.jsligo -m SFTStorage -o build/SFTStorage.tz 
	ligo compile contract src/SFTStorage.jsligo -m SFTStorage -o build/SFTStorage.json  --michelson-format json 

build-storage-factory: build-nft-storage build-sft-storage
	@echo "Building storage-deployer" 
	ligo compile contract src/StorageFactory.jsligo -m StorageFactory -o build/StorageFactory.tz 
	ligo compile contract src/StorageFactory.jsligo -m StorageFactory -o build/StorageFactory.json --michelson-format json

build-collection-factories: build-nft-storage build-sft-storage
	@echo "Building nft collection deployer" 
	ligo compile contract src/NFTCollectionFactory.jsligo -m NFTCollectionFactory -o build/NFTCollectionFactory.tz 
	ligo compile contract src/NFTCollectionFactory.jsligo -m NFTCollectionFactory -o build/NFTCollectionFactory.json --michelson-format json
	@echo "Building sft collection deployer" 
	ligo compile contract src/SFTCollectionFactory.jsligo -m SFTCollectionFactory -o build/SFTCollectionFactory.tz 
	ligo compile contract src/SFTCollectionFactory.jsligo -m SFTCollectionFactory -o build/SFTCollectionFactory.json --michelson-format json

build-tokens:
	@echo "Building tokens..."
	@echo "Building nft -"
	ligo compile contract src/NFTv2.jsligo -m NFT -o build/NFT.tz 
	ligo compile contract src/NFTv2.jsligo -m NFT -o build/NFT.json --michelson-format json
	@echo "Building sft -"
	ligo compile contract src/SFTv2.jsligo -m SFT -o build/SFT.tz 
	ligo compile contract src/SFTv2.jsligo -m SFT -o build/SFT.json --michelson-format json

build-bridge: build-storage-factory build-collection-factories
	@echo "Building bridge -"
	ligo compile contract src/Bridge.jsligo -m Bridge -o build/Bridge.tz 
	ligo compile contract src/Bridge.jsligo -m Bridge -o build/Bridge.json --michelson-format json

build: build-bridge

clean: 
	rm -rf ./build/*