import Crypto "mo:ed25519/crypto/crypto";
import Text "mo:base/Text";
import Principal "mo:base/Principal";

module {
    public type BlacklistValidator = {
        public_key: Text;
        principal: Principal
    };

    public func hash(c : BlacklistValidator) : Blob {
        let encoded = to_candid (c);
        let hashed = Crypto.fromBlob(#sha512, encoded);
        hashed;
    };
};
