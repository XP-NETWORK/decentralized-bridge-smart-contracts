import "@stdlib/deploy";
import "./message.tact";
message UnlockToken {
    to: Address;
}

contract NFTStorageERC721 {
    owner: Address;
    collectionAddress: Address;
    init(collectionAddress: Address, bridgeAddress: Address){
        self.owner = bridgeAddress;
        self.collectionAddress = collectionAddress;
    }

    fun onlyOwner(sender: Address) {
        require(sender == self.owner, "Only the owner can call this function");
    }

    receive(msg: StorageDeploy){
        send(SendParameters{
                value: ton("0.07"),
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
                    sourceChain: msg.sourceChain
                }.toCell()
            }
        );
    }

    receive("DepositTokens"){
        send(SendParameters{
                value: ton("0.07"),
                to: self.collectionAddress,
                bounce: true,
                mode: SendIgnoreErrors,
                body: Transfer{
                    query_id: 0,
                    new_owner: myAddress(),
                    response_destination: myAddress(),
                    custom_payload: emptyCell(),
                    forward_amount: 0,
                    forward_payload: emptySlice()
                }.toCell()
            }
        );
    }

    receive(msg: UnlockToken){
        self.onlyOwner(context().sender);
        send(SendParameters{
                value: ton("0.07"),
                to: self.collectionAddress,
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