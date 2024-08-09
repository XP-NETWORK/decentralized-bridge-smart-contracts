import "@stdlib/deploy";
import "./message.tact";
import "./nftErc721Storage.tact";

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
        let msgValue: Int = (context().value - (minTonsForStorage + gasConsumption));
        let nftStorageERC721Init: StateInit = initOf NFTStorageERC721(msg.collectionAddress, self.owner);
        let nftStorageERC721Address: Address = contractAddress(nftStorageERC721Init);
        send(SendParameters{
                value: msgValue, // pay for message and potential deployment
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
                    nftItemAddress: msg.nftItemAddress,
                    metaDataUri: msg.metaDataUri,
                    sender: msg.sender,
                }.toCell()
            }
        );
    }
}