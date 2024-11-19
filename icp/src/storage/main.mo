import Principal "mo:base/Principal";
import Text "mo:base/Text";
import ExperimentalCycles "mo:base/ExperimentalCycles";
import Error "mo:base/Error";
import ICRC7 "mo:icrc7-mo";
import Service "mo:icrc7-mo/service";

shared actor class Storage(
    _args : {
        owner : Principal;
        collection : Text;
    }
) {
    stable var owner = _args.owner;
    stable var collection = _args.collection;

    public shared (msg) func unlock_token(token_id : Nat, receiver : ICRC7.Account): async () {
        assert msg.caller == owner;
        let nft = actor (collection) : Service.Service;
        var ta : Service.TransferArg = {
            to = receiver;
            token_id = token_id;
            created_at_time = null;
            memo = null;
            subaccount = null;
            from_subaccount = null;
        };
        let response = (await nft.icrc7_transfer([ta]))[0];
        switch (response) {
            case (?value) {
                switch (value) {
                    case (#Ok(_)) {};
                    case (#Err(error)) {
                        throw Error.reject("Storage: Failed to unlock token" # debug_show error);
                    };
                };
            };
            case (null) {
                throw Error.reject("Storage: Got no response for transfer call");
            };
        };
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
