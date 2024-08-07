import "@stdlib/deploy";
import "./message.tact";
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
    contractAddress: Slice;
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
    sourceNftContractAddress: Slice; // Address of the NFT contract in the source chain
    royaltyReceiver: Address; // address of user who is going to receive royalty
    metadata: Cell; // Metadata related to the NFT being transferred
}
struct ClaimData4 {
    newContent: Cell;
    transactionHash: String; // Transaction hash of the transfer on the source chain
    royalty: RoyaltyParams; // royalty of nft collection
    lockTxChain: String;
}

struct ClaimData {
    data1: ClaimData1;
    data2: ClaimData2;
    data3: ClaimData3;
    data4: ClaimData4;
}
struct Token {
    tokenId: Int;
    chain: String;
    contractAddress: Slice;
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
}
message Lock721 {
    tokenId: Int as uint256;
    destinationChain: Cell;
    destinationUserAddress: Cell;
    sourceNftContractAddress: Address;
    metaDataUri: Cell;
}
message ClaimNFT721 {
    data: ClaimData;
    signatures: map<Int, SignerAndSignature>;
    len: Int as uint256;
}

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
    destinationChain: Cell; // Chain to where the NFT is being transferred
    destinationUserAddress: Cell; // User's address in the destination chain
    sourceNftContractAddress: Slice; // Address of the NFT contract in the source chain
    tokenAmount: Int as uint256; // Token amount is 1 incase it is ERC721
    nftType: String; // NFT type is either 721 or 1155.
    sourceChain: String; // Name of the chain emitting
    metaDataUri: Cell;
}
message UnLock721Event {
    to: Address;
    tokenId: Int as uint256;
    contractAddress: Address;
}
message ClaimedEvent {
    lockTxChain: String;
    tokenId: Int as uint256;
    newlyDeployCollection: Address;
    sourceChain: String; // Chain from where the NFT is being transferred
    transactionHash: String; // Transaction hash of the transfer on the source chain
}

contract Bridge {

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

    fun checkStorage721(storageMapping721: map<Int,Address>, sourceNftContractAddress: Address, tokenId: Int, destinationChain: Cell, destinationUserAddress: Cell, sourceNftContractAddressLock: Slice, sourceChain: String, isOriginal: Bool, nftItemAddress: Address, metaDataUri: Cell, msgValue: Int, sender: Address) {
        let key: Int = beginCell().storeAddress(sourceNftContractAddress).storeSlice(self.selfChain.asSlice()).endCell().hash();
        let storageAddress: Address? = storageMapping721.get(key);
        if(storageAddress == null){
            send(SendParameters{
                    value: msgValue,
                    to: self.storageDeployer,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: DeployNFT721Storage{
                        collectionAddress: sourceNftContractAddress,
                        isOriginal: isOriginal,
                        key: key,
                        tokenId: tokenId, 
                        destinationChain: destinationChain, 
                        destinationUserAddress: destinationUserAddress, 
                        sourceNftContractAddressLock: sourceNftContractAddressLock, 
                        sourceChain: sourceChain,
                        nftItemAddress: nftItemAddress,
                        metaDataUri: metaDataUri,
                        sender: sender,
                    }.toCell()
                }
            );
        }
        else{
            self.transferToStorage721(sourceNftContractAddress, storageAddress!!, tokenId, destinationChain, destinationUserAddress, sourceNftContractAddressLock, sourceChain, nftItemAddress, metaDataUri, msgValue, sender);
        }
    }

    fun transferToStorage721(sourceNftContractAddress: Address, storageAddress: Address, tokenId: Int, destinationChain: Cell, destinationUserAddress: Cell, sourceNftContractAddressLock: Slice, sourceChain: String, nftItemAddress: Address, metaDataUri: Cell, msgValue: Int, sender: Address) {
        send(SendParameters{
                value: msgValue,
                to: nftItemAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                body: Transfer{
                    query_id: 0,
                    new_owner: storageAddress,
                    response_destination: sender,
                    custom_payload: emptyCell(),
                    forward_amount: 0,
                    forward_payload: emptySlice()
                }.toCell()
            }
        );

        emit(LockedEvent{
                    tokenId: tokenId,
                    destinationChain: destinationChain,
                    destinationUserAddress: destinationUserAddress,
                    sourceNftContractAddress: sourceNftContractAddressLock,
                    tokenAmount: 1,
                    nftType: self.TYPEERC721,
                    sourceChain: sourceChain,
                    metaDataUri: metaDataUri,
                }.toCell());
    }

    fun verifySignature(hash: Int, sigs: map<Int, SignerAndSignature>, len: Int): ValidatorsToRewards {
        let counter: Int = 0;
        let percentage: Int = 0;
        let validatorsToReward: map<Int, Address> = emptyMap();
        let validatorsKeys: map<Int, Int> = emptyMap();
        let requireUnique: map<Int, Bool> = emptyMap();

        repeat(len) {
            let sig: SignerAndSignature = sigs.get(counter)!!;
            let alreadySigned: Bool? = requireUnique.get(sig.key);
            if (alreadySigned == null) {
                requireUnique.set(sig.key, true);
                let ok: Bool = checkSignature(hash, sig.signature, sig.key);
                let vv: Validator = self.Validator(sig.key)!!;
                if(ok && vv.added){
                    validatorsToReward.set(percentage, vv.address);
                    validatorsKeys.set(percentage, sig.key);
                    percentage = percentage + 1;
                }
                counter = counter + 1;
            }
        }

        require(percentage >= (((self.validatorsCount * 2)/3)+1),"Threshold not reached!");

        return ValidatorsToRewards{addresses: validatorsToReward, publicKeys: validatorsKeys, len: percentage};
    }

    fun rewardValidators(fee: Int, validatorsToRewards: ValidatorsToRewards){
        let counter: Int = 0;
        require(fee > 0, "Invalid fees");

        let totalRewards: Int = myBalance();

        require(totalRewards >= fee, "No rewards available");

        let feePerValidator: Int = fee / validatorsToRewards.len;
        repeat(validatorsToRewards.len){
            let pk: Int = validatorsToRewards.publicKeys.get(counter)!!;
            let v: Validator = self.Validator(pk)!!;
            v.pendingRewards = (v.pendingRewards + feePerValidator);
            self.validators.set(pk, v);

            counter = counter + 1;
        }
    }

    fun unlock721(to: Address, tokenId: Int, contractAddress: Address, msgValue: Int){
        // let nftStorageERC721Init: StateInit = initOf NFTStorageERC721(contractAddress, myAddress());
        send(SendParameters{
                value: msgValue,
                to: contractAddress,
                mode: SendIgnoreErrors,
                bounce: true,
                body: UnlockToken{
                    to: to,
                    token_id: tokenId
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

        require(length > 0, "Must have signatures!");
        let percentage: Int = 0;
        let counter: Int = 0;
        let requireUnique: map<Int, Bool> = emptyMap();

        repeat(length) {
            let sig: SignerAndSignature = msg.sigs.get(counter)!!;
            let alreadySigned: Bool? = requireUnique.get(sig.key);
            if (alreadySigned == null) {
                requireUnique.set(sig.key, true);
                let op_hash: Int = msg.newValidatorPublicKey.toCell().hash();
                let ok: Bool = checkSignature(op_hash, sig.signature, sig.key);
                let v: Validator? = self.Validator(sig.key);
                if(ok && v != null){
                    if((v!!).added){
                    percentage = percentage + 1;
                    }
                }
                counter = counter + 1;
            }
        }

        require(percentage >= (((self.validatorsCount * 2)/3)+1),"Threshold not reached!");

        self.validators.set(msg.newValidatorPublicKey.key, Validator{address: msg.newValidatorAddress, added: true, pendingRewards: 0});
        self.validatorsCount = (self.validatorsCount + 1);

        emit(AddNewValidatorEvent{validator: msg.newValidatorPublicKey.key}.toCell());
    }

    receive(msg: RewardValidator){
        let v_o: Validator? = self.Validator(msg.validator.key);
        require(v_o != null,"Validator does not exist!");
        let v: Validator = v_o!!;
        let reward: Int = v.pendingRewards;
        v.pendingRewards = 0;
        self.validators.set(msg.validator.key,v);

        require(reward < (myBalance() - context().value - minTonsForStorage - gasConsumption), "Insufficient balance");
        send(SendParameters{
            to: v.address,
            bounce: true,
            value: reward,
            mode: SendIgnoreErrors
        });
    }

    receive(msg: HiFromDeployNFT721Storage){
        let sender: Address = contractAddress(initOf NFTStorageERC721(msg.sourceNftContractAddress, myAddress()));
        let msgValue: Int = (context().value - (minTonsForStorage + gasConsumption));
        require(sender == context().sender, "Only Storage Contract can call");
        if(msg.isOriginal){
            self.originalStorageMapping721.set(msg.key, msg.storageAddress);
        }
        else{
            self.duplicateStorageMapping721.set(msg.key, msg.storageAddress);
        }

        self.transferToStorage721(msg.sourceNftContractAddress, msg.storageAddress, msg.tokenId, msg.destinationChain, msg.destinationUserAddress, msg.sourceNftContractAddressLock, msg.sourceChain, msg.nftItemAddress, msg.metaDataUri, msgValue, msg.sender);
    }

    receive(msg: HiFromDeployNFT721Collection){
        require(context().sender == self.collectionDeployer, "Only collection deployer can call this message");
        emit(ClaimedEvent{ lockTxChain: msg.lockTxChain, tokenId: msg.tokenId, newlyDeployCollection: msg.newlyDeployCollection, sourceChain: msg.sourceChain, transactionHash: msg.transactionHash}.toCell());
    }

    receive(oa: OwnershipAssigned){
        let msg: Lock721 = Lock721 {
            tokenId: oa.forward_payload.loadInt(256),
            sourceNftContractAddress: oa.forward_payload.loadAddress(),
            destinationChain: oa.forward_payload.loadRef(),
            destinationUserAddress: oa.forward_payload.loadRef(),
            metaDataUri: oa.forward_payload.loadRef(),
        };
        let sender: Address = oa.prev_owner;
        let msgValue: Int = (context().value - (minTonsForStorage + gasConsumption));
        let nftItemAddress: Address = context().sender;
        let sCollectionAddress: Address = msg.sourceNftContractAddress;
        let isOriginal: Bool = false;
        let isTokenExists: Token? = self.tokenInfoKeySelf.get(sCollectionAddress);

        let tokenId: Int = 0;

        if(isTokenExists != null){
            let isTokenExistsNotNull: Token = isTokenExists!!;
            tokenId = isTokenExistsNotNull.tokenId;
        }
        else{
            tokenId = msg.tokenId;

            let tokenKey: Int = beginCell().storeInt(msg.tokenId, 64).storeSlice(self.selfChain.asSlice()).storeSlice(msg.sourceNftContractAddress.asSlice()).endCell().hash();

            self.tokenInfoKeySelf.set(sCollectionAddress, Token{tokenId: msg.tokenId, chain: self.selfChain, contractAddress: msg.sourceNftContractAddress.asSlice()});

            self.tokenInfoGetSelf.set(tokenKey, sCollectionAddress);
        }

        // Check if sourceNftContractAddress is original or duplicate
        let originalCollectionInfo: DuplicateToOriginalContractInfo? = self.duplicateToOriginalMapping.get(sCollectionAddress);
        let originalCollectionInfoNotNull: DuplicateToOriginalContractInfo = DuplicateToOriginalContractInfo{ keyChain: "", chain: "", contractAddress: emptySlice(), lastIndex: 0, collectionContent: emptyCell() };
        
        if(originalCollectionInfo != null){
            originalCollectionInfoNotNull = originalCollectionInfo!!;
        }
        else{
            isOriginal = true;
        }
        if(self.convertStringToCell(originalCollectionInfoNotNull.keyChain) == self.selfChainCell) {
            // isOriginal
            self.checkStorage721(self.originalStorageMapping721, sCollectionAddress, tokenId, msg.destinationChain, msg.destinationUserAddress, originalCollectionInfoNotNull.contractAddress, originalCollectionInfoNotNull.chain, isOriginal, nftItemAddress, msg.metaDataUri, msgValue, sender);
        }
        else if(isOriginal){
            self.checkStorage721(self.originalStorageMapping721, sCollectionAddress, tokenId, msg.destinationChain, msg.destinationUserAddress, msg.sourceNftContractAddress.asSlice(), self.selfChain, true, nftItemAddress, msg.metaDataUri, msgValue, sender);
        }
        else{
            // notOriginal
            self.checkStorage721(self.duplicateStorageMapping721, sCollectionAddress, tokenId, msg.destinationChain, msg.destinationUserAddress, msg.sourceNftContractAddress.asSlice(), self.selfChain, false, nftItemAddress, msg.metaDataUri, msgValue, sender);
        }
    }

    receive(msg: ClaimNFT721){
        let sourceNftContractAddressSlice: Slice = msg.data.data3.sourceNftContractAddress;
        self.hasCorrectFee(msg.data.data3.fee);
        self.matchesCurrentChain(msg.data.data1.destinationChain);
        let msgValue: Int = (context().value - (msg.data.data3.fee + minTonsForStorage + gasConsumption));
        require(self.convertStringToCell(msg.data.data2.nftType) == self.convertStringToCell(self.TYPEERC721), "Invalid NFT type!");
        let hash: Int = msg.data.toCell().hash();
        require(self.uniqueIdentifier.get(hash) == null,"Data already processed!");
        self.uniqueIdentifier.set(hash, true);
        let validatorsToReward: ValidatorsToRewards = self.verifySignature(hash, msg.signatures, msg.len);
        self.rewardValidators(msg.data.data3.fee, validatorsToReward);
        
        let key1: Int = beginCell().storeSlice(msg.data.data3.sourceNftContractAddress).storeSlice(msg.data.data1.sourceChain.asSlice()).endCell().hash();

        let duplicateCollectionAddress: OriginalToDuplicateContractInfo? = self.originalToDuplicateMapping.get(key1);
        let duplicateCollectionAddressNotNull: OriginalToDuplicateContractInfo = OriginalToDuplicateContractInfo{ keyChain: "", chain: "", contractAddress: myAddress(), lastIndex: 0, collectionContent: emptyCell() };

        let hasDuplicate: Bool = false;
        let hasStorage: Bool = false;
        let storageContract: Address? = null;
        let lastCollectionAddress: Address? = null;
        let lastTokenId: Int? = null;
        let tokenKey: Int = beginCell().storeInt(msg.data.data1.tokenId, 64).storeSlice(msg.data.data1.sourceChain.asSlice()).storeSlice(msg.data.data3.sourceNftContractAddress).endCell().hash();
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
                self.unlock721(msg.data.data1.destinationUserAddress, msg.data.data1.tokenId, storageContract!!, msgValue);
                emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: storageContract!!, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
            }
            else{
                send(SendParameters{
                    value: msgValue,
                    to: duplicateCollectionAddressNotNull.contractAddress,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: Mint{token_id: msg.data.data1.tokenId, owner: msg.data.data1.destinationUserAddress, content: msg.data.data3.metadata}.toCell(),
                    }
                );
                let nftStateInit: StateInit = initOf NftItem(duplicateCollectionAddressNotNull.contractAddress, duplicateCollectionAddressNotNull.lastIndex + 1, self.collectionDeployer, duplicateCollectionAddressNotNull.collectionContent);
                let nftAddress: Address = contractAddress(nftStateInit);
                self.tokenInfoKeySelf.set(nftAddress, Token{tokenId: msg.data.data1.tokenId, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress});
                self.tokenInfoGetSelf.set(tokenKey, nftAddress);
                lastCollectionAddress = duplicateCollectionAddressNotNull.contractAddress;
                lastTokenId = duplicateCollectionAddressNotNull.lastIndex + 1;
                emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: duplicateCollectionAddressNotNull.contractAddress, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
            }
        }
        else if (hasDuplicate && !hasStorage){
            send(SendParameters{
                    value: msgValue,
                    to: duplicateCollectionAddressNotNull.contractAddress,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: Mint{token_id: msg.data.data1.tokenId, owner: msg.data.data1.destinationUserAddress, content: msg.data.data3.metadata}.toCell(),
                    
                    }
                );
                let nftStateInit: StateInit = initOf NftItem(duplicateCollectionAddressNotNull.contractAddress, duplicateCollectionAddressNotNull.lastIndex + 1, self.collectionDeployer, duplicateCollectionAddressNotNull.collectionContent);
                let nftAddress: Address = contractAddress(nftStateInit);
                self.tokenInfoKeySelf.set(nftAddress, Token{tokenId: msg.data.data1.tokenId, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress});
                self.tokenInfoGetSelf.set(tokenKey, nftAddress);
                
                lastCollectionAddress = duplicateCollectionAddressNotNull.contractAddress;
                lastTokenId = duplicateCollectionAddressNotNull.lastIndex + 1;
                emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: duplicateCollectionAddressNotNull.contractAddress, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
        }
        else if (!hasDuplicate && !hasStorage){
                send(SendParameters{
                    value: msgValue,
                    to: self.collectionDeployer,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body: DeployNFT721Collection{
                        collection_content: msg.data.data4.newContent,
                        royalty_params: msg.data.data4.royalty,
                        destination_user_address: msg.data.data1.destinationUserAddress,
                        source_chain: msg.data.data1.sourceChain,
                        transaction_hash: msg.data.data4.transactionHash,
                        metadata: msg.data.data3.metadata,
                        token_id: msg.data.data1.tokenId,
                        lockTxChain: msg.data.data4.lockTxChain
                    }.toCell()
                    }
                );

                let collectionInit: StateInit = initOf NftCollection(myAddress(), msg.data.data4.newContent, msg.data.data4.royalty);
                let collectionAddress: Address = contractAddress(collectionInit);

                // update duplicate mappings

                let nftStateInit: StateInit = initOf NftItem(collectionAddress, 0, myAddress(), msg.data.data4.newContent);

                let nftAddress: Address = contractAddress(nftStateInit);

                self.originalToDuplicateMapping.set(key1, OriginalToDuplicateContractInfo{keyChain: msg.data.data1.sourceChain, chain: self.selfChain, contractAddress: collectionAddress, lastIndex: 0, collectionContent: emptyCell()});
                
                // self.originalToDuplicateMapping.set(key1, OriginalToDuplicateContractInfo{keyChain: msg.data.data1.sourceChain, chain: self.selfChain, contractAddress: nftAddress, lastIndex: 0, collectionContent: emptyCell()});

                self.duplicateToOriginalMapping.set(collectionAddress, DuplicateToOriginalContractInfo{keyChain: self.selfChain, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress, lastIndex: 0, collectionContent: emptyCell()});
                
                // self.duplicateToOriginalMapping.set(nftAddress, DuplicateToOriginalContractInfo{keyChain: self.selfChain, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress, lastIndex: 0, collectionContent: emptyCell()});

                self.tokenInfoKeySelf.set(nftAddress, Token{tokenId: msg.data.data1.tokenId, chain: msg.data.data1.sourceChain, contractAddress: msg.data.data3.sourceNftContractAddress});
                self.tokenInfoGetSelf.set(tokenKey, nftAddress);
                emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: collectionAddress, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
        }
        else if (!hasDuplicate && hasStorage){
            let originalCollection: Address = sourceNftContractAddressSlice.loadAddress();

            lastCollectionAddress = originalCollection;
            lastTokenId = msg.data.data1.tokenId;

            if(isTokenExists != null){
               self.unlock721(msg.data.data1.destinationUserAddress, msg.data.data1.tokenId, storageContract!!, msgValue);
               emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: originalCollection, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
            }
            
            else{
                // ============= This could be wrong. Need verification ============
                 send(SendParameters{
                    value: msgValue,
                    to: originalCollection,
                    mode: SendIgnoreErrors,
                    bounce: true,
                    body:  Mint{token_id: msg.data.data1.tokenId, owner: msg.data.data1.destinationUserAddress, content: msg.data.data3.metadata}.toCell(),

                    }
                );
                 emit(ClaimedEvent{lockTxChain: msg.data.data4.lockTxChain, tokenId: msg.data.data1.tokenId, newlyDeployCollection: originalCollection, sourceChain: msg.data.data1.sourceChain, transactionHash: msg.data.data4.transactionHash}.toCell());
            }
            // ============= This could be wrong. Need verification ============
        }
        else{
            require(false, "Invalid bridge state");
        }
    }

    get fun Original721Mapping(sourceNftContractAddress: Address, sourceChain: String): Address?{
        let key: Int = beginCell().storeAddress(sourceNftContractAddress).storeSlice(sourceChain.asSlice()).endCell().hash();
        return self.originalStorageMapping721.get(key);
    }

    get fun Duplicate721Mapping(contractAddress: Address): Address?{
        let key: Int = beginCell().storeAddress(contractAddress).storeSlice(self.selfChain.asSlice()).endCell().hash();
        return self.duplicateStorageMapping721.get(key);
    }

    get fun OriginalToDuplicate(sourceNftContractAddress: String, sourceChain: String): OriginalToDuplicateContractInfo? {
        let key: Int = beginCell().storeSlice(sourceNftContractAddress.asSlice()).storeSlice(sourceChain.asSlice()).endCell().hash();
        return self.originalToDuplicateMapping.get(key);
    }

    get fun DuplicateToOriginal(key: Address): DuplicateToOriginalContractInfo? {
        return self.duplicateToOriginalMapping.get(key);
    }

    get fun TokenInfo(key: Address): Token? {
        return self.tokenInfoKeySelf.get(key);
    }

    get fun TokenInfoSelf(tokenId: Int, sourceChain: String, sourceNftContractAddress: Slice): Address? {
        let tokenKey: Int = beginCell().storeInt(tokenId, 64).storeSlice(sourceChain.asSlice()).storeSlice(sourceNftContractAddress).endCell().hash();
        return self.tokenInfoGetSelf.get(tokenKey);
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

    receive() {
        
    }
}