import Text "mo:base/Text";
import Principal "mo:base/Principal";
import ExperimentalCycles "mo:base/ExperimentalCycles";
import Error "mo:base/Error";
import Storage "../storage/main";

shared actor class StorageFactory() {
    stable var owner: ?Principal = null;
    public shared (msg) func deploy_storage(collection : Text) : async Principal {

        switch owner {
            case (null) {
                throw Error.reject("CollectionFactory: Contract Not Initialized.");
            };
            case (?value) {
                if (msg.caller != value) {
                    throw Error.reject("StorageFactory: Invalid caller. Only owner can call this method");
                };
                ExperimentalCycles.add<system>(1_000_000_000_000);
                let store = await Storage.Storage({
                    collection = collection;
                    owner = value;
                });
                return Principal.fromActor(store);
            };
        };

    };

    public query func get_owner() : async ?Principal {
        return owner;
    };

    public func set_owner(new_owner : Principal) : async () {
        if (owner != null) {
            throw Error.reject("CollectionFactory: Owner already set.");
        };
        owner := ?new_owner;
    };

    //Internal cycle management - good general case
    public func acceptCycles() : async () {
        let available = ExperimentalCycles.available();
        let accepted = ExperimentalCycles.accept<system>(available);
        assert (accepted == available);
    };

    public query func availableCycles() : async Nat {
        return ExperimentalCycles.balance();
    };
};
