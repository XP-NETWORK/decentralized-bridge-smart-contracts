import "@stdlib/deploy";
import "./message.tact";
import "./nftErc721.tact";

message DeployNFT721Collection {
    owner_address: Address;
    collection_content: Cell;
    royalty_params: RoyaltyParams;
}
message CreatedCollection {
    collectionAddress: Address;
}

contract NFTCollectionDeployer {
    owner: Address;
    init(bridgeAddress: Address){
        self.owner = bridgeAddress;
    }

     fun onlyOwner(sender: Address) {
        require(sender == self.owner, "Only owner can call");
    }

    receive("CollectionDeployer"){}

    receive(msg: DeployNFT721Collection){
        self.onlyOwner(context().sender);
        let collectionInit: StateInit = initOf NftCollection(myAddress(),
            msg.collection_content,
            msg.royalty_params
        );
        let collectionAddress: Address = contractAddress(collectionInit);
        send(SendParameters{
                value: ton("0.08"), // pay for message and potential deployment
                to: collectionAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                code: collectionInit.code, // if child is not deployed, also deploy it
                data: collectionInit.data,
                body: "CollectionDeploy".asComment()
            }
        );
        // emit(CreatedCollection{collectionAddress: collectionAddress}.toCell());
        self.reply(HiFromDeployNFT721Collection{collectionAddress: collectionAddress}.toCell());
    }
}