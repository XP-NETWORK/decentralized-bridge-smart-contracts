import Hash "mo:base/Hash";
import Blob "mo:base/Blob";
import Crypto "mo:ed25519/crypto/crypto";

module {
    public type OriginalToDuplicateMappingKey = {
        source_nft_contract_address : Text;
        source_chain : Text;
    };
    public func hash(k : OriginalToDuplicateMappingKey) : Hash.Hash {
        let encoded = to_candid (k);
        let hashed = Crypto.fromBlob(#sha512, encoded);
        Blob.hash(hashed);
    };

    public func equal(k1 : OriginalToDuplicateMappingKey, k2 : OriginalToDuplicateMappingKey) : Bool {
        return k1.source_nft_contract_address == k2.source_nft_contract_address and k1.source_chain == k2.source_chain;
    };
};
