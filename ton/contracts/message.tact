message(0x693d3950) GetRoyaltyParams {
    query_id: Int as uint64;
}
message(0xa8cb00ad) ReportRoyaltyParams {
    query_id: Int as uint64;
    numerator: Int as uint16;
    denominator: Int as uint16;
    destination: Address;
}
struct CollectionData {
    next_item_index: Int;
    collection_content: Cell;
    owner_address: Address;
}
struct RoyaltyParams {
    numerator: Int;
    denominator: Int;
    destination: Address;
}
message(0x5fcc3d14) Transfer {
    query_id: Int as uint64;
    new_owner: Address;
    response_destination: Address;
    custom_payload: Cell?;
    forward_amount: Int as coins;
    forward_payload: Slice;
}
message(0x05138d91) OwnershipAssigned {
    query_id: Int as uint64;
    prev_owner: Address;
    forward_payload: Slice;
}
message(0xd53276db) Excesses {
    query_id: Int as uint64;
}
message(0x2fcb26a2) GetStaticData {
    query_id: Int as uint64;
}
message(0x8b771735) ReportStaticData {
    query_id: Int as uint64;
    index_id: Int;
    collection: Address;
}
struct GetNftData {
    is_initialized: Bool;
    index: Int;
    collection_address: Address;
    owner_address: Address;
    individual_content: Cell;
}
message HiFromDeployNFT721Storage {
    sourceNftContractAddress: Address;
    storageAddress: Address;
    isOriginal: Bool;
    key: Int;
    tokenId: Int;
    destinationChain: Cell;
    destinationUserAddress: Cell;
    sourceNftContractAddressLock: Slice;
    sourceChain: String;
    nftItemAddress: Address;
    metaDataUri: Cell;
}

message HiFromDeployNFT721Collection{
    tokenId: Int as uint256;
    newlyDeployCollection: Address;
    sourceChain: String;
    transactionHash: String;
    lockTxChain: String;
}
message CollectionDeploy {
    newOwner: Address;
    metadata: Cell;
    token_id: Int;
}
message StorageDeploy {
    sourceNftContractAddress: Address;
    isOriginal: Bool;
    key: Int;
    tokenId: Int;
    destinationChain: Cell;
    destinationUserAddress: Cell;
    sourceNftContractAddressLock: Slice;
    sourceChain: String;
    nftItemAddress: Address;
    metaDataUri: Cell;
}