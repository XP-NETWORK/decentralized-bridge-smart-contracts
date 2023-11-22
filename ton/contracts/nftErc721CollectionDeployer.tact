import "@stdlib/deploy";
import "./message.tact";
import "./nftErc721.tact";
message DeployNFT721Collection {
    collection_content: Cell;
    royalty_params: RoyaltyParams;
    destination_user_address: Address;
    source_chain: String;
    source_nft_contract_address: String;
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
        let collectionInit: StateInit = initOf NftCollection(self.owner, msg.collection_content, msg.royalty_params);
        let collectionAddress: Address = contractAddress(collectionInit);
        send(SendParameters{
                value: ton("1.8"), // pay for message and potential deployment
                to: collectionAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                code: collectionInit.code, // if child is not deployed, also deploy it
                data: collectionInit.data,
                body: CollectionDeploy{newOwner: msg.destination_user_address}.toCell()
            }
        );
        // emit(CreatedCollection{collectionAddress: collectionAddress}.toCell());
        // self.reply(HiFromDeployNFT721Collection{
        //         collectionAddress: collectionAddress,
        //         collectionContent: msg.collection_content,
        //         destinationUserAddress: msg.destination_user_address,
        //         sourceChain: msg.source_chain,
        //         sourceNftContractAddress: msg.source_nft_contract_address
        //     }.toCell()
        // );
    }
}