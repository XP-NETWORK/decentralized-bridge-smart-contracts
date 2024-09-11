build-bridge:
	@echo "Building bridge..."
	cd bridge && cargo near build --no-docker

build-storage:
	@echo "Building bridge..."
	cd storage && cargo near build --no-docker

build-storage-factory:
	@echo "Building bridge..."
	cd storage-factory && cargo near build --no-docker

build-nft:
	@echo "Building All NFT..."
	cd nft && cargo near build --no-docker

build-collection-factory:
	@echo "Building collection-factory..."
	cd collection-factory && cargo near build --no-docker

build-all: build-nft build-storage build-storage-factory build-collection-factory build-bridge


build:
	@echo "Building Contracts ..."
	@make build-all

clean:
	@echo "Cleaning..."
	cargo clean