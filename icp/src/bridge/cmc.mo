import Text "mo:base/Text";
import Nat "mo:base/Nat";
import Principal "mo:base/Principal";

module {
    
    public type BlockIndex = Nat64;

    type NotifyError = {
        #Refunded : {
            reason : Text;
            block_index : ?BlockIndex;
        };
        #Processing;
        #TransactionTooOld : BlockIndex;
        #InvalidTransaction : Text;
        #Other : { error_code : Nat64; error_message : Text };
    };

    type NotifyTopUpResult = {
      #Ok : Nat;
      #Err : NotifyError;
    };

    type NotifyTopUpArg = {
      block_index : BlockIndex;
      canister_id : Principal;
    };

    type NotifyCreateCanisterResult = {
      #Ok : Principal;
      #Err : NotifyError;
    };

    type NotifyCreateCanisterArg = {
      block_index : Nat64;
      controller : Principal;
    };

    public type CMC = actor{
        notify_top_up : (NotifyTopUpArg) -> async (NotifyTopUpResult);
        notify_create_canister : (NotifyCreateCanisterArg) -> async (NotifyCreateCanisterResult);
    };
}