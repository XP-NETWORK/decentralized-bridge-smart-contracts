import Crypto "mo:ed25519/crypto/crypto";
import Nat64 "mo:base/Nat64";
module {
    public type ClaimData = {
        nft_type : Text;
        token_amount : Nat;
        name : Text;
        symbol : Text;
        source_nft_contract_address : Text;
        token_id : Nat;
        destination_chain : Text;
        destination_user_address : Principal;
        royalty : Nat;
        royalty_receiver : Principal;
        metadata : Text;
        fee : Nat64;
        transaction_hash : Text;
        source_chain : Text;
    };

    public func hash(c : ClaimData) : Blob {
        let encoded = to_candid (c);
        let hashed = Crypto.fromBlob(#sha512, encoded);
        hashed;
    };
};
