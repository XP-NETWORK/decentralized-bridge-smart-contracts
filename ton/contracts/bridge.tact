import "@stdlib/deploy";
import "./nftErc721StorageDeployer.tact";
import "./nftErc721CollectionDeployer.tact";
struct Validator {
    address: Address;
    added: Bool;
    pendingRewards: Int as coins;
}
struct SignerAndSignature {
    signature: Slice;
    key: Int as uint256;
}
struct NewValidator {
    key: Int as uint256;
}
struct ValidatorsToRewards {
    addresses: map<Int, Address>;
    publicKeys: map<Int, Int>;
    len: Int;
}
struct DuplicateToOriginalContractInfo {
    keyChain: String;
    chain: String;
    contractAddress: String;
    lastIndex: Int;
    collectionContent: Cell;
}
struct OriginalToDuplicateContractInfo {
    keyChain: String;
    chain: String;
    contractAddress: Address;
    lastIndex: Int;
    collectionContent: Cell;
}
struct ClaimData1 {
    tokenId: Int as uint64; // Unique ID for the NFT transfer
    sourceChain: String; // Chain from where the NFT is being transferred
    destinationChain: String; // Chain to where the NFT is being transferred
    destinationUserAddress: Address; // User's address in the destination chain
    tokenAmount: Int as uint64; // Number of NFTs being transferred
}
struct ClaimData2 {
    name: String; // name of NFT collection
    symbol: String; // symbol of nft collection
    nftType: String; // Type of the NFT (could be ERC721 or ERC1155)
}
struct ClaimData3 {
    fee: Int as uint64; // fee that needs to be paid by the user to the bridge,
    sourceNftContractAddress: String; // Address of the NFT contract in the source chain
    royaltyReceiver: Address; // address of user who is going to receive royalty
    metadata: String; // Metadata related to the NFT being transferred
}
struct ClaimData4 {
    newContent: Cell;
    transactionHash: String; // Transaction hash of the transfer on the source chain
    royalty: RoyaltyParams; // royalty of nft collection
}

// struct ClaimData4{

//}

struct ClaimData {
    data1: ClaimData1;
    data2: ClaimData2;
    data3: ClaimData3;
    data4: ClaimData4;
}
struct Token {
    tokenId: Int;
    chain: String;
    contractAddress: String;
}

// MESSAGES
message AddValidator {
    newValidatorPublicKey: NewValidator;
    newValidatorAddress: Address;
    sigs: map<Int, SignerAndSignature>;
    len: Int as uint256;
}
message RewardValidator {
    validator: NewValidator;
    sigs: map<Int, SignerAndSignature>;
    len: Int as uint256;
}
message Lock721 {
    tokenId: Int as uint256;
    destinationChain: String;
    destinationUserAddress: String;
    sourceNftContractAddress: String;
}
message ClaimNFT721 {
    data: ClaimData;
    signatures: map<Int, SignerAndSignature>;
    len: Int as uint256;
}
// struct Transfer {
//     seqno: Int as uint32;
//     mode: Int as uint8;
//     to: Address;
//     amount: Int as coins;
//     body: Cell?;
// }
// message(123) TransferMessage {
//     signature: Slice;
//     transfer: Transfer;
//     key: Int as uint256;
// }

// EVENTS
message StakeEvent {
    amount: Int as coins;
    asd: String;
}
message AddNewValidatorEvent {
    validator: Int as uint256;
}
message RewardValidatorEvent {
    validator: Int as uint256;
}
message LockedEvent {
    tokenId: Int as uint256; // Unique ID for the NFT transfer
    destinationChain: String; // Chain to where the NFT is being transferred
    destinationUserAddress: String; // User's address in the destination chain
    sourceNftContractAddress: String; // Address of the NFT contract in the source chain
    tokenAmount: Int as uint256; // Token amount is 1 incase it is ERC721
    nftType: String; // NFT type is either 721 or 1155.
    sourceChain: String; // Name of the chain emitting
}
message UnLock721Event {
    to: Address;
    tokenId: Int as uint256;
    contractAddress: Address;
}
message ClaimedEvent {
    sourceChain: String; // Chain from where the NFT is being transferred
    transactionHash: String; // Transaction hash of the transfer on the source chain
}
message Gando {
    content: Cell;
    royalty: RoyaltyParams;
}

contract Bridge {
    const MinTonForStorage: Int = ton("0.01"); // enough for 1 KB of storage for 2.5 years

    validators: map<Int, Validator>;

    collections: map<Int, Address>;

    uniqueIdentifier: map<Int, Bool>;

    validatorsCount: Int as uint256 = 0;

    originalToDuplicateMapping: map<Int,OriginalToDuplicateContractInfo>;

    duplicateToOriginalMapping: map<Address,DuplicateToOriginalContractInfo>;

    originalStorageMapping721: map<Int,Address>;

    duplicateStorageMapping721: map<Int,Address>;

    tokenInfoGetSelf: map<Int, Address>;

    tokenInfoKeySelf: map<Address, Token>;

    selfChain: String = "";
    selfChainCell: Cell;
    TYPEERC721: String = "singular";

    collectionDeployer: Address;
    storageDeployer: Address;

    fun convertStringToCell(string: String): Cell {
        let builder: StringBuilder = beginString();
        builder.append(string);
        return builder.toCell();
    }
    fun matchesCurrentChain(destinationChain: String) {
        require(self.convertStringToCell(destinationChain) == self.selfChainCell, "Invalid destination chain!");
    }

    fun hasCorrectFee(fee: Int) {
        let ctx: Context = context();
        require(ctx.value >= fee,"data.fee LESS THAN sent amount!");
    }

    fun checkStorage721(storageMapping721: map<Int,Address>, sourceNftContractAddress: Address, tokenId: Int, isOriginal: Bool) {
        let key: Int = beginCell().storeAddress(sourceNftContractAddress).storeSlice(self.selfChain.asSlice()).endCell().hash();
        let storageAddress: Address? = storageMapping721.get(key);
        if(storageAddress == null){
            send(SendParameters{
                    value: ton("0.07"),
                    to: self.storageDeployer,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: DeployNFT721Storage{
                        collectionAddress: sourceNftContractAddress,
                        isOriginal: isOriginal,
                        key:key
                    }.toCell()
                }
            );
        }
        else{
            self.transferToStorage721(sourceNftContractAddress, storageAddress!!);
        }
    }

    fun transferToStorage721(sourceNftContractAddress: Address, storageAddress: Address) {
        send(SendParameters{
                value: ton("0.07"),
                to: sourceNftContractAddress,
                mode: SendIgnoreErrors,
                bounce: false,
                body: Transfer{
                    query_id: 0,
                    new_owner: storageAddress,
                    response_destination: context().sender,
                    custom_payload: emptyCell(),
                    forward_amount: 0,
                    forward_payload: emptySlice()
                }.toCell()
            }
        );
    }

    fun verifySignature(hash: Int, sigs: map<Int, SignerAndSignature>, len: Int): ValidatorsToRewards {
        let counter: Int = 0;
        let percentage: Int = 0;
        let validatorsToReward: map<Int, Address> = emptyMap();
        let validatorsKeys: map<Int, Int> = emptyMap();

        repeat(len) {
            let sig: SignerAndSignature = sigs.get(counter)!!;
            let ok: Bool = checkSignature(hash, sig.signature, sig.key);
            let vv: Validator = self.Validator(sig.key)!!;
            if(ok && vv.added){
                validatorsToReward.set(percentage, vv.address);
                validatorsKeys.set(percentage, sig.key);
                percentage = percentage + 1;
            }
            counter = counter + 1;
        }

        require(percentage >= (((self.validatorsCount * 2)/3)+1),"Threshold not reached!");

        return ValidatorsToRewards{addresses: validatorsToReward, publicKeys: validatorsKeys, len: percentage};
    }

    fun rewardValidators(fee: Int, validatorsToRewards: ValidatorsToRewards){
        let counter: Int = 0;
        require(fee > 0, "Invalid fees");

        let totalRewards: Int = myBalance();

        require(totalRewards >= fee, "No rewards available");

        let feePerValidator: Int = totalRewards / validatorsToRewards.len;

        repeat(validatorsToRewards.len){
            let pk: Int = validatorsToRewards.publicKeys.get(counter)!!;
            let v: Validator = self.Validator(pk)!!;
            v.pendingRewards = v.pendingRewards + feePerValidator;
            counter = counter + 1;
        }
    }

    fun unlock721(to: Address, tokenId: Int, contractAddress: Address){
        let nftStorageERC721Init: StateInit = initOf NFTStorageERC721(contractAddress, myAddress());
        send(SendParameters{
                value: ton("0.08"),
                to: contractAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                body: UnlockToken{
                    to: to
                }.toCell()
            }
        );

        emit(UnLock721Event{to: to,tokenId: tokenId,contractAddress: contractAddress}.toCell());
    }

    init(validatorPublicKey: Int, validatorAddress: Address, chainType: String){
        self.validators.set(validatorPublicKey, Validator{address: validatorAddress, added: true, pendingRewards: 0});
        self.validatorsCount = (self.validatorsCount + 1);
        self.selfChain = chainType;
        let builder: StringBuilder = beginString();
        builder.append(chainType);
        self.selfChainCell = builder.toCell();
        self.collectionDeployer = myAddress();
        self.storageDeployer = myAddress();
    }

    receive(msg: Excesses){

    }
    receive("Deploy"){
        dump("LUN");
        // emit(StakeEvent{amount: ton("0.007"),asd: "gando"}.toCell());
        // Do nothing

        let collectionDeployerInit: StateInit = initOf NFTCollectionDeployer(myAddress());
        let collectionDeployerAddress: Address = contractAddress(collectionDeployerInit);
        send(SendParameters{
                value: ton("0.07"), // pay for message and potential deployment
                to: collectionDeployerAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                code: collectionDeployerInit.code, // if child is not deployed, also deploy it
                data: collectionDeployerInit.data,
                body: "CollectionDeployer".asComment()
            }
        );

        let storageDeployerInit: StateInit = initOf NFTStorageDeployer(myAddress());
        let storageDeployerAddress: Address = contractAddress(storageDeployerInit);
        send(SendParameters{
                value: ton("0.07"), // pay for message and potential deployment
                to: storageDeployerAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                code: storageDeployerInit.code, // if child is not deployed, also deploy it
                data: storageDeployerInit.data,
                body: "StorageDeployer".asComment()
            }
        );

        self.collectionDeployer = contractAddress(collectionDeployerInit);
        self.storageDeployer = contractAddress(storageDeployerInit);
    }

    receive(msg: AddValidator){

        let length: Int = msg.len;

        require(length > 0,"Must have signatures!");
        let percentage: Int = 0;
        let counter: Int = 0;

        repeat(length) {
            let sig: SignerAndSignature = msg.sigs.get(counter)!!;
            let op_hash: Int = msg.newValidatorPublicKey.toCell().hash();
            let ok: Bool = checkSignature(op_hash, sig.signature, sig.key);
            let v: Validator = self.Validator(sig.key)!!;
            if(ok && v.added){
                percentage = percentage + 1;
            }
            counter = counter + 1;
        }

        require(percentage >= (((self.validatorsCount * 2)/3)+1),"Threshold not reached!");

        self.validators.set(msg.newValidatorPublicKey.key, Validator{address: msg.newValidatorAddress, added: true, pendingRewards: 0});
        self.validatorsCount = (self.validatorsCount + 1);

        emit(AddNewValidatorEvent{validator: msg.newValidatorPublicKey.key}.toCell());
    }

    receive(msg: RewardValidator){
        let length: Int = msg.len;

        require(length > 0,"Must have signatures!");
        let v: Validator = self.Validator(msg.validator.key)!!;
        require(v.added,"Validator does not exist!");

        let percentage: Int = 0;
        let counter: Int = 0;

        repeat(length) {
            let sig: SignerAndSignature = msg.sigs.get(counter)!!;
            let op_hash: Int = msg.validator.toCell().hash();
            let ok: Bool = checkSignature(op_hash, sig.signature, sig.key);
            let vv: Validator = self.Validator(sig.key)!!;
            if(ok && vv.added){
                percentage = percentage + 1;
            }
            counter = counter + 1;
        }

        require(percentage >= (((self.validatorsCount * 2)/3)+1),"Threshold not reached!");
        let vvv: Validator = self.Validator(msg.validator.key)!!;
        let reward: Int = vvv.pendingRewards;
        vvv.pendingRewards = 0;
        self.validators.set(msg.validator.key,v);

        let amount: Int = min(reward, myBalance() - context().value - self.MinTonForStorage);
        require(amount > 0, "Insufficient balance");
        // send(SendParameters{
        //     to: address(msg.validator.key.toString()),
        //     bounce: true,
        //     value: amount,
        //     mode: SendRemainingValue + SendIgnoreErrors
        // });
    }

    receive(msg: HiFromDeployNFT721Storage){
        // dump("😑 handling hi from deploy nft 721 storage");
        // dump(msg.storageAddress);
        if(msg.isOriginal){
            self.originalStorageMapping721.set(msg.key, msg.storageAddress);
        }
        else{
            self.duplicateStorageMapping721.set(msg.key, msg.storageAddress);
        }

        self.transferToStorage721(msg.sourceNftContractAddress, msg.storageAddress);
    }

    receive(msg: HiFromDeployNFT721Collection){
        // self.collections.set(1,msg.collectionAddress);
        // dump("😑 handling hi from deploy nft 721 collection");
        // dump(msg.collectionAddress);
        // send(SendParameters{
        //             value: ton("0.08"),
        //             to: msg.collectionAddress,
        //             mode: SendIgnoreErrors,
        //             bounce: true,
        //             body: "Mint".asComment()
        //             }
        //         );

        // // update duplicate mappings

        // let key1: Int = beginCell().storeSlice(msg.sourceNftContractAddress.asSlice()).storeSlice(msg.sourceChain.asSlice()).endCell().hash();
        // self.originalToDuplicateMapping.set(key1, OriginalToDuplicateContractInfo{keyChain: msg.sourceChain, chain: self.selfChain, contractAddress: msg.collectionAddress, lastIndex: 0, collectionContent: msg.collectionContent});

        // self.duplicateToOriginalMapping.set(msg.collectionAddress, DuplicateToOriginalContractInfo{keyChain: self.selfChain, chain: msg.sourceChain, contractAddress: msg.sourceNftContractAddress, lastIndex: 0, collectionContent: msg.collectionContent});

        // let nftStateInit: StateInit = initOf NftItem(msg.collectionAddress, 0, self.collectionDeployer, msg.collectionContent);
        // let nftAddress: Address = contractAddress(nftStateInit);

        // send(SendParameters{
        //             value: ton("0.07"),
        //             to: nftAddress,
        //             mode: SendIgnoreErrors,
        //             bounce: false,
        //             body: Transfer{
        //                 query_id: 0,
        //                 new_owner: msg.destinationUserAddress,
        //                 response_destination: context().sender,
        //                 custom_payload: emptyCell(),
        //                 forward_amount: 0,
        //                 forward_payload: emptySlice()
        //             }.toCell()
        //             }
        //         );
    }

    receive(msg: Lock721){
        let slice: Slice = msg.sourceNftContractAddress.asSlice();
        let sNftAddress: Address = slice.loadAddress();
        let isTokenExists: Token? = self.tokenInfoKeySelf.get(sNftAddress);
        let tokenId: Int = 0;
        if(isTokenExists != null){
            let isTokenExistsNotNull: Token = isTokenExists!!;
            tokenId = isTokenExistsNotNull.tokenId;
        }
        else{
            tokenId = msg.tokenId;

            let tokenKey: Int = beginCell().storeInt(msg.tokenId, 64).storeSlice(self.selfChain.asSlice()).storeSlice(msg.sourceNftContractAddress.asSlice()).endCell().hash();

            self.tokenInfoKeySelf.set(sNftAddress, Token{tokenId: msg.tokenId, chain: self.selfChain, contractAddress: msg.sourceNftContractAddress});

            self.tokenInfoGetSelf.set(tokenKey, sNftAddress);
        }

        // Check if sourceNftContractAddress is original or duplicate
        let originalCollectionInfo: DuplicateToOriginalContractInfo? = self.duplicateToOriginalMapping.get(sNftAddress);
        let originalCollectionInfoNotNull: DuplicateToOriginalContractInfo = DuplicateToOriginalContractInfo{ keyChain: "", chain: "", contractAddress: "", lastIndex: 0, collectionContent: emptyCell() };
        
        if(originalCollectionInfo != null){
            originalCollectionInfoNotNull = originalCollectionInfo!!;
        }

        if(self.convertStringToCell(originalCollectionInfoNotNull.keyChain) == self.selfChainCell) {
            // isOriginal
            self.checkStorage721(self.originalStorageMapping721, sNftAddress, msg.tokenId, true);

            emit(LockedEvent{
                    tokenId: tokenId,
                    destinationChain: msg.destinationChain,
                    destinationUserAddress: msg.destinationUserAddress,
                    sourceNftContractAddress: originalCollectionInfoNotNull.contractAddress,
                    tokenAmount: 1,
                    nftType: self.TYPEERC721,
                    sourceChain: originalCollectionInfoNotNull.chain
                }.toCell());
        }else{
            // notOriginal
            self.checkStorage721(self.duplicateStorageMapping721, sNftAddress, msg.tokenId, false);

            emit(LockedEvent{
                    tokenId: tokenId,
                    destinationChain: msg.destinationChain,
                    destinationUserAddress: msg.destinationUserAddress,
                    sourceNftContractAddress: msg.sourceNftContractAddress,
                    tokenAmount: 1,
                    nftType: self.TYPEERC721,
                    sourceChain: self.selfChain
                }.toCell());
        }
    }

    receive(msg: ClaimNFT721){
        let slice: Slice = msg.data.data3.sourceNftContractAddress.asSlice();
        let sNftAddress: Address = slice.loadAddress();
        // self.hasCorrectFee(msg.data.fee);
        self.matchesCurrentChain(msg.data.data1.destinationChain);
        require(self.convertStringToCell(msg.data.data2.nftType) == self.convertStringToCell(self.TYPEERC721), "Invalid NFT type!");
        let hash: Int = msg.data.toCell().hash();
        require(self.uniqueIdentifier.get(hash) == null,"Data already processed!");
        self.uniqueIdentifier.set(hash, true);
        let validatorsToReward: ValidatorsToRewards = self.verifySignature(hash, msg.signatures, msg.len);
        self.rewardValidators(msg.data.data3.fee, validatorsToReward);
        let key1: Int = beginCell().storeSlice(msg.data.data3.sourceNftContractAddress.asSlice()).storeSlice(msg.data.data1.sourceChain.asSlice()).endCell().hash();

        let duplicateCollectionAddress: OriginalToDuplicateContractInfo? = self.originalToDuplicateMapping.get(key1);
        let duplicateCollectionAddressNotNull: OriginalToDuplicateContractInfo = OriginalToDuplicateContractInfo{ keyChain: "", chain: "", contractAddress: myAddress(), lastIndex: 0, collectionContent: emptyCell() };

        let hasDuplicate: Bool = false;
        let hasStorage: Bool = false;
        let storageContract: Address? = myAddress();
        let tokenKey: Int = beginCell().storeInt(msg.data.data1.tokenId, 64).storeSlice(msg.data.data1.sourceChain.asSlice()).storeSlice(msg.data.data3.sourceNftContractAddress.asSlice()).endCell().hash();
        let isTokenExists: Address? = self.tokenInfoGetSelf.get(tokenKey);
        if(duplicateCollectionAddress != null){
            hasDuplicate = true;
            duplicateCollectionAddressNotNull = duplicateCollectionAddress!!;
        }
        if(hasDuplicate){
            let key2: Int = beginCell().storeAddress(duplicateCollectionAddressNotNull.contractAddress).storeSlice(self.selfChain.asSlice()).endCell().hash();
            storageContract = self.duplicateStorageMapping721.get(key2);
        }
        else{
            storageContract = self.originalStorageMapping721.get(key1);
        }
        if(storageContract != null){
            hasStorage = true;
        }
        if(hasDuplicate && hasStorage){
            if(isTokenExists != null){
                self.unlock721(msg.data.data1.destinationUserAddress, msg.data.data1.tokenId, storageContract!!);
            }
            else{
                send(SendParameters{
                    value: ton("0.07"),
                    to: duplicateCollectionAddressNotNull.contractAddress,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: "Mint".asComment()
                    }
                );
                let nftStateInit: StateInit = initOf NftItem(duplicateCollectionAddressNotNull.contractAddress, duplicateCollectionAddressNotNull.lastIndex + 1, self.collectionDeployer, duplicateCollectionAddressNotNull.collectionContent);
                let nftAddress: Address = contractAddress(nftStateInit);
                self.tokenInfoKeySelf.set(nftAddress, Token{tokenId: msg.data.data1.tokenId, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress});
                self.tokenInfoGetSelf.set(tokenKey, nftAddress);
                send(SendParameters{
                    value: ton("0.07"),
                    to: nftAddress,
                    mode: SendIgnoreErrors,
                    bounce: false,
                    body: Transfer{
                        query_id: 0,
                        new_owner: msg.data.data1.destinationUserAddress,
                        response_destination: context().sender,
                        custom_payload: emptyCell(),
                        forward_amount: 0,
                        forward_payload: emptySlice()
                    }.toCell()
                    }
                );
            }
        }
        else if (hasDuplicate && !hasStorage){
            send(SendParameters{
                    value: ton("0.07"),
                    to: duplicateCollectionAddressNotNull.contractAddress,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: "Mint".asComment()
                    }
                );
                let nftStateInit: StateInit = initOf NftItem(duplicateCollectionAddressNotNull.contractAddress, duplicateCollectionAddressNotNull.lastIndex + 1, self.collectionDeployer, duplicateCollectionAddressNotNull.collectionContent);
                let nftAddress: Address = contractAddress(nftStateInit);
                self.tokenInfoKeySelf.set(nftAddress, Token{tokenId: msg.data.data1.tokenId, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress});
                self.tokenInfoGetSelf.set(tokenKey, nftAddress);
                send(SendParameters{
                    value: ton("0.07"),
                    to: nftAddress,
                    mode: SendIgnoreErrors,
                    bounce: false,
                    body: Transfer{
                        query_id: 0,
                        new_owner: msg.data.data1.destinationUserAddress,
                        response_destination: context().sender,
                        custom_payload: emptyCell(),
                        forward_amount: 0,
                        forward_payload: emptySlice()
                    }.toCell()
                    }
                );
        }
        else if (!hasDuplicate && !hasStorage){
                send(SendParameters{
                    value: ton("2"),
                    to: self.collectionDeployer,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: DeployNFT721Collection{
                        collection_content: msg.data.data4.newContent,
                        royalty_params: msg.data.data4.royalty,
                        destination_user_address: msg.data.data1.destinationUserAddress,
                        source_chain: msg.data.data1.sourceChain,
                        source_nft_contract_address: msg.data.data3.sourceNftContractAddress
                    }.toCell()
                    }
                );



        // let collectionInit: StateInit = initOf NftCollection(myAddress(), msg.data.data4.newContent, msg.data.data4.royalty);
        // let collectionAddress: Address = contractAddress(collectionInit);

        // send(SendParameters{
        //             value: ton("2"),
        //             to: collectionAddress,
        //             mode: SendIgnoreErrors,
        //             bounce: false,
        //             body: "Mint".asComment()
        //             }
        //         );

        // update duplicate mappings

        // self.originalToDuplicateMapping.set(key1, OriginalToDuplicateContractInfo{keyChain: msg.data.data1.sourceChain, chain: self.selfChain, contractAddress: collectionAddress, lastIndex: 0, collectionContent: emptyCell()});

        // self.duplicateToOriginalMapping.set(collectionAddress, DuplicateToOriginalContractInfo{keyChain: self.selfChain, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress, lastIndex: 0, collectionContent: emptyCell()});

        // let nftStateInit: StateInit = initOf NftItem(collectionAddress, 0, myAddress(), msg.data.data4.newContent);
        // let nftAddress: Address = contractAddress(nftStateInit);

        // send(SendParameters{
        //             value: ton("0.08"),
        //             to: nftAddress,
        //             mode: SendIgnoreErrors,
        //             bounce: false,
        //             body: Transfer{
        //                 query_id: 0,
        //                 new_owner: msg.data.data1.destinationUserAddress,
        //                 response_destination: context().sender,
        //                 custom_payload: emptyCell(),
        //                 forward_amount: 0,
        //                 forward_payload: emptySlice()
        //             }.toCell()
        //             }
        //         );
        }
        // else if (!hasDuplicate && hasStorage){
        //     let originalCollection: Address = sNftAddress;
        //     if(isTokenExists != null){
        //        self.unlock721(msg.data.destinationUserAddress, msg.data.tokenId, storageContract);
        //     }
        //     else{
        //         // ============= This could be wrong. Need verification ============
        //          send(SendParameters{
        //             value: ton("0.08"),
        //             to: originalCollection,
        //             mode: SendIgnoreErrors,
        //             bounce: true,
        //             body: "Mint".asComment()
        //             }
        //         );
        //     }
        //     // ============= This could be wrong. Need verification ============
        // }
        else{
            require(false, "Invalid bridge state");
        }
        // emit(ClaimedEvent{sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
    }

    get fun OriginalToDuplicate(sourceNftContractAddress: Address, sourceChain: String): OriginalToDuplicateContractInfo? {
        let key: Int = beginCell().storeSlice(sourceNftContractAddress.asSlice()).storeSlice(sourceChain.asSlice()).endCell().hash();
        return self.originalToDuplicateMapping.get(key);
    }

    get fun DuplicateToOriginal(key: Address): DuplicateToOriginalContractInfo? {
        return self.duplicateToOriginalMapping.get(key);
    }

    get fun TokenInfo(key: Address): Token? {
        return self.tokenInfoKeySelf.get(key);
    }

    get fun Validator(key: Int): Validator? {
        return self.validators.get(key);
    }

    get fun ValidatorsCount(): Int? {
        return self.validatorsCount;
    }

    get fun CollectionDeployer(): Address? {
        return self.collectionDeployer;
    }

    get fun StorageDeployer(): Address? {
        return self.storageDeployer;
    }

    get fun Collections(key: Int): Address?{
        return self.collections.get(key);
    }
}