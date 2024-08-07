import "./message.tact";
import "@stdlib/deploy";

contract NftCollection {
    next_item_index: Int as uint32 = 0;
    owner_address: Address;
    royalty_params: RoyaltyParams;
    collection_content: Cell;
    init(owner_address: Address, collection_content: Cell, royalty_params: RoyaltyParams){
        self.owner_address = owner_address;
        self.collection_content = collection_content;
        self.royalty_params = royalty_params;
    }

    receive(msg: CollectionDeploy){
        let ctx: Context = context(); // get sender Info
        let msgValue: Int = ctx.value;
        let tonBalanceBeforeMsg: Int = (myBalance() - msgValue);
        let storageFee: Int = (minTonsForStorage - min(tonBalanceBeforeMsg, minTonsForStorage));
        msgValue = (msgValue - (storageFee + gasConsumption));
        self.mint(msg.newOwner, msgValue, msg.metadata, msg.token_id);
    }

    receive(msg: Mint){
        let ctx: Context = context(); // get sender Info
        let msgValue: Int = ctx.value;
        let tonBalanceBeforeMsg: Int = (myBalance() - msgValue);
        let storageFee: Int = (minTonsForStorage - min(tonBalanceBeforeMsg, minTonsForStorage));
        msgValue = (msgValue - (storageFee + gasConsumption));
        self.mint(msg.owner, msgValue, msg.content, msg.token_id);
    }

    // ===== Private Methods ===== //
     fun mint(sender: Address, msgValue: Int, content: Cell, tid: Int) {
        require(self.next_item_index >= 0, "non-sequential NFTs");
        let nft_init: StateInit = self.getNftItemInit(tid);
        send(SendParameters{
                to: contractAddress(nft_init),
                value: msgValue,
                bounce: false,
                mode: SendIgnoreErrors,
                body: MintOne{
                    new_owner: sender,
                    content: content,
                }.toCell(),
                code: nft_init.code,
                data: nft_init.data
            }
        );
        self.next_item_index = (self.next_item_index + 1);
    }

    receive(msg: GetRoyaltyParams){
        let ctx: Context = context(); // get sender Info
        send(SendParameters{
                to: ctx.sender,
                value: 0,
                mode: 64,
                bounce: false,
                body: ReportRoyaltyParams{
                    query_id: msg.query_id,
                    numerator: self.royalty_params.numerator,
                    denominator: self.royalty_params.denominator,
                    destination: self.owner_address
                }.toCell()
            }
        );
    }

    // ------------------ Get Function  ------------------ //
    get fun get_collection_data(): CollectionData {
        return
            CollectionData{
                next_item_index: self.next_item_index,
                collection_content: self.collection_content,
                owner_address: self.owner_address
            };
    }

    get fun get_nft_address_by_index(item_index: Int): Address? {
        let initCode: StateInit = self.getNftItemInit(item_index);
        return contractAddress(initCode);
    }

    get fun getNftItemInit(item_index: Int): StateInit {
        return initOf NftItem(myAddress(), item_index, self.owner_address, self.collection_content);
    }

    get fun get_nft_content(index: Int, individual_content: Cell): Cell {
        let b: StringBuilder = beginString();
        let ic: String = individual_content.asSlice().asString();
        b.append(ic);
        return b.toCell();
    }

    get fun royalty_params(): RoyaltyParams {
        return self.royalty_params;
    }
}

contract NftItem {
    collection_address: Address;
    item_index: Int;
    owner: Address;
    individual_content: Cell;
    is_initialized: Bool;
    init(collection_address: Address, item_index: Int, owner: Address, individual_content: Cell){
        self.collection_address = collection_address;
        self.item_index = item_index;
        self.owner = collection_address;
        self.individual_content = individual_content;
        self.is_initialized = false;
    }

     receive(msg: MintOne) {
        require(context().sender == self.collection_address, "not from collection");
        let msgValue: Int = context().value; // Check the gasCost for storage
        let tonBalanceBeforeMsg: Int = (myBalance() - msgValue);
        let storageFee: Int = (minTonsForStorage - min(tonBalanceBeforeMsg, minTonsForStorage));
        msgValue = (msgValue - (storageFee + gasConsumption));
        require(!self.is_initialized, "already initialized");
        self.is_initialized = true;
        self.owner = msg.new_owner;
        self.individual_content = msg.content;
        send(SendParameters{
            to: msg.new_owner,
            value: msgValue,
            mode: SendPayGasSeparately,
            body: Excesses { query_id: 420 }.toCell()
        });
    }

    receive(msg: Transfer){
        let ctx: Context = context();
        // Only Owner of the this NFT Item can transfer it.
        require(ctx.sender == self.owner, "not owner");
        let msgValue: Int = ctx.value; // Check the gasCost for storage
        let tonBalanceBeforeMsg: Int = (myBalance() - msgValue);
        let storageFee: Int = (minTonsForStorage - min(tonBalanceBeforeMsg, minTonsForStorage));
        msgValue = (msgValue - (storageFee + gasConsumption));
        if (self.is_initialized == false) {
            // Initial Transfer, aka the "Minting" of the NFT
            self.is_initialized = true;
            self.owner = msg.new_owner;
            send(SendParameters{
                    to: msg.response_destination,
                    value: 0,
                    mode: (SendIgnoreErrors + SendRemainingValue),
                    body: Excesses{query_id: msg.query_id}.toCell()
                }
            );
        } else {
            self.owner = msg.new_owner; // change current owner to the new_owner
            if (msg.forward_amount > 0) {
                send(SendParameters{
                        to: msg.new_owner,
                        value: msg.forward_amount,
                        mode: SendIgnoreErrors,
                        bounce: false,
                        body: OwnershipAssigned{
                            query_id: msg.query_id,
                            prev_owner: ctx.sender,
                            forward_payload: msg.forward_payload
                        }.toCell()
                    }
                );
            }
            msgValue = (msgValue - ctx.readForwardFee());
            if (msg.response_destination != null) {
                send(SendParameters{
                        to: msg.response_destination,
                        value: (msgValue - msg.forward_amount),
                        mode: SendPayGasSeparately,
                        body: Excesses{query_id: msg.query_id}.toCell()
                    }
                );
            }
        }
    }

    receive(msg: GetStaticData){
        let ctx: Context = context();
        send(SendParameters{
                to: ctx.sender,
                value: 0,
                mode: 64, // (return msg amount except gas fees)
                bounce: true,
                body: ReportStaticData{
                    query_id: msg.query_id,
                    index_id: self.item_index,
                    collection: self.collection_address
                }.toCell()
            }
        );
    }

    // --------- Get Function  --------- //
    get fun get_nft_data(): GetNftData {
        return
            GetNftData{
                is_initialized: self.is_initialized,
                index: self.item_index,
                collection_address: self.collection_address,
                owner_address: self.owner,
                individual_content: self.individual_content
            };
    }
}


message MintOne {
    new_owner: Address;
    content: Cell;
}

message Mint {
    token_id: Int;
    owner: Address;
    content: Cell;
}