import "@stdlib/deploy";
import "./nftErc721Storage.tact";
message DeployNFT721Storage {
    collectionAddress: Address;
    isOriginal: Bool;
    key: Int;
    tokenId: Int;
    destinationChain: Cell;
    destinationUserAddress: Cell;
    sourceNftContractAddressLock: Slice;
    sourceChain: String;
    nftItemAddress: Address;
}

contract NFTStorageDeployer {
    owner: Address;
    init(bridgeAddress: Address){
        self.owner = bridgeAddress;
    }

    fun onlyOwner(sender: Address) {
        require(sender == self.owner, "Only owner can call");
    }

    receive("StorageDeployer"){}

    receive(msg: DeployNFT721Storage){
        self.onlyOwner(context().sender);
        let nftStorageERC721Init: StateInit = initOf NFTStorageERC721(msg.collectionAddress, self.owner);
        let nftStorageERC721Address: Address = contractAddress(nftStorageERC721Init);
        send(SendParameters{
                value: ton("0.2"), // pay for message and potential deployment
                to: nftStorageERC721Address,
                mode: SendIgnoreErrors,
                bounce: true,
                code: nftStorageERC721Init.code, // if child is not deployed, also deploy it
                data: nftStorageERC721Init.data,
                body: StorageDeploy{
                    sourceNftContractAddress: msg.collectionAddress,
                    isOriginal: msg.isOriginal,
                    key: msg.key,
                    tokenId: msg.tokenId,
                    destinationChain: msg.destinationChain,
                    destinationUserAddress: msg.destinationUserAddress,
                    sourceNftContractAddressLock: msg.sourceNftContractAddressLock,
                    sourceChain: msg.sourceChain,
                    nftItemAddress: msg.nftItemAddress
                }.toCell()
            }
        );
    }
}