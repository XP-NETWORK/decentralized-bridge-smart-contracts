import "@stdlib/deploy";

message UnlockToken {
    to: Address;
}

contract NFTStorageERC721 {
    owner: Address;
    collectionAddress: Address;
    init(collectionAddress: Address, owner: Address){
        self.owner = owner;
        self.collectionAddress = collectionAddress;
    }

     fun onlyOwner(sender: Address) {
        require(sender == self.owner, "Only the owner can call this function");
    }

    receive("StorageDeploy"){}

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