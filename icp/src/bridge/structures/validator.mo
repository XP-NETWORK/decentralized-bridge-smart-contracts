import Nat64 "mo:base/Nat64";

module {

    public type Validator = {
        address : Principal;
        pending_rewards : Nat64;
    };

};
