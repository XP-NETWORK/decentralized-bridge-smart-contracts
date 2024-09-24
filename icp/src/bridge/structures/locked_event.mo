import Hash "mo:base/Hash";
import Blob "mo:base/Blob";
import Crypto "mo:ed25519/crypto/crypto";
module {
    public type LockedEvent = {
        source_nft_contract_address : Principal;
        source_chain : Text;
        destination_chain : Text;
        destination_user_address : Text;
        token_id : Nat;
        nft_type : Text;
        token_amount : Nat;
        metadata_uri : Text;
        sender_address: Text;
    };

    public func hash(ev : LockedEvent) : Hash.Hash {
        let encoded = to_candid (ev);
        let hashed = Crypto.fromBlob(#sha512, encoded);
        Blob.hash(hashed);
    };
};
