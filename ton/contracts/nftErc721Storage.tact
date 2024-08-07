import "@stdlib/deploy";
import "./message.tact";
message UnlockToken {
    to: Address;
    token_id: Int;
}

contract NFTStorageERC721 {
    owner: Address;
    collectionAddress: Address;
    nftItemAddresses: map<Int, Address>;
    init(collectionAddress: Address, bridgeAddress: Address){
        self.owner = bridgeAddress;
        self.collectionAddress = collectionAddress;
    }

    fun onlyOwner(sender: Address) {
        require(sender == self.owner, "Only the owner can call this function");
    }

    receive(msg: StorageDeploy){
        let msgValue: Int = (context().value - (minTonsForStorage + gasConsumption));
        self.nftItemAddresses.set(msg.tokenId, msg.nftItemAddress);
        send(SendParameters{
                value: msgValue,
                to: self.owner,
                bounce: true,
                mode: SendIgnoreErrors,
                body: HiFromDeployNFT721Storage{
                    sourceNftContractAddress: msg.sourceNftContractAddress,
                    storageAddress: myAddress(),
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
    
    receive(msg: Excesses) {
        
    }

    receive(msg: UnlockToken){
        self.onlyOwner(context().sender);
        let msgValue: Int = (context().value - (minTonsForStorage + gasConsumption));
        let nftItemAddress: Address = self.nftItemAddresses.get(msg.token_id)!!;
        send(SendParameters{
                value: msgValue,
                to: nftItemAddress,
                bounce: true,
                mode: SendIgnoreErrors,
                body: Transfer{
                    query_id: 0,
                    new_owner: msg.to,
                    response_destination: msg.to,
                    custom_payload: emptyCell(),
                    forward_amount: 0,
                    forward_payload: emptySlice()
                }.toCell()
            }
        );
    }
}