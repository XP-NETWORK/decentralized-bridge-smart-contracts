import Text "mo:base/Text";
import Principal "mo:base/Principal";
import ExperimentalCycles "mo:base/ExperimentalCycles";
import Error "mo:base/Error";
import Option "mo:base/Option";
import NFT "../nft/main";
import Icrc7 "../nft/initial_state/icrc7";

shared actor class CollectionFactory() {
    private stable var owner : ?Principal = null;
    public shared (msg) func deploy_nft_collection(name : Text, symbol : Text) : async Principal {
        switch owner {
            case (null) {
                throw Error.reject("CollectionFactory: Contract Not Initialized.");
            };
            case (?value) {
                if (msg.caller != value) {
                    throw Error.reject("CollectionFactory: Invalid caller. Only owner can call this method");
                };
                ExperimentalCycles.add<system>(1_000_000_000_000);
                let collection = await NFT.NFT({
                    icrc7_args = ?Icrc7.defaultConfig(value, Option.make(name), Option.make(symbol));
                    icrc37_args = null;
                    icrc3_args = ?{
                        maxActiveRecords = 4000;
                        settleToRecords = 2000;
                        maxRecordsInArchiveInstance = 5_000_000;
                        maxArchivePages = 62500; //allows up to 993 bytes per record
                        archiveIndexType = #Stable;
                        maxRecordsToArchive = 10_000;
                        archiveCycles = 2_000_000_000_000; //two trillion
                        archiveControllers = null;
                        supportedBlocks = [];
                    };
                });
                return Principal.fromActor(collection);
            };
        };
    };

    public query func get_owner() : async ?Principal {
        return owner;
    };
    public shared func set_owner(new : Principal) : async () {
        owner := ?new;
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
