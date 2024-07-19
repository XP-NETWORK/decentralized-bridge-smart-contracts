import Text "mo:base/Text";
import Principal "mo:base/Principal";
import Nat "mo:base/Nat";
import Hash "mo:base/Hash";
import Blob "mo:base/Blob";
import Crypto "mo:ed25519/crypto/crypto";

module {
    public type ClaimedEvent = {
        source_chain : Text;
        nft_contract : Principal;
        token_id : Nat;
        transaction_hash : Text;
    };

    public func hash(c : ClaimedEvent) : Hash.Hash {
        let encoded = to_candid (c);
        let hashed = Crypto.fromBlob(#sha512, encoded);
        Blob.hash(hashed);
    };
};
