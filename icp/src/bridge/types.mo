import Nat "mo:base/Nat";
module {

    public type ICP = {
        e8s : Nat64;
    };

    public type Timestamp = {
        timestamp_nanos: Nat64;
    };

    public type AccountIdentifier = Blob;

    public type SubAccount = Blob;

    public type BlockIndex = Nat64;

    public type Memo = Nat64;

    public type TransferArgs = {
        memo: Memo;
        amount: ICP;
        fee: ICP;
        from_subaccount: ?SubAccount;
        to: AccountIdentifier;
        created_at_time: ?Timestamp;
    };

    public type TransferError = {
        #BadFee : { expected_fee : ICP; };
        #InsufficientFunds : { balance: ICP; };
        #TxTooOld : { allowed_window_nanos: Nat64 };
        #TxCreatedInFuture : Null;
        #TxDuplicate : { duplicate_of: BlockIndex; };
    };

    public type TransferResult = {
        #Ok : BlockIndex;
        #Err : TransferError;
    };

    public type Icrc1Timestamp = Nat64;

    public type Icrc1Tokens = Nat;

    public type Icrc1BlockIndex = Nat;

    public type Account = {
        owner : Principal;
        subaccount : ?SubAccount;
    };
    
    public type TransferFromArgs = {
        spender_subaccount : ?SubAccount;
        from : Account;
        to : Account;
        amount : Icrc1Tokens;
        fee : ?Icrc1Tokens;
        memo : ?Blob;
        created_at_time: ?Icrc1Timestamp;
    };

    public type TransferFromError = {
        #BadFee : { expected_fee : Icrc1Tokens };
        #BadBurn : { min_burn_amount : Icrc1Tokens };
        #InsufficientFunds : { balance : Icrc1Tokens };
        #InsufficientAllowance : { allowance : Icrc1Tokens };
        #TooOld;
        #CreatedInFuture : { ledger_time : Icrc1Timestamp };
        #Duplicate : { duplicate_of : Icrc1BlockIndex };
        #TemporarilyUnavailable;
        #GenericError : { error_code : Nat; message : Text };
    };

    public type TransferFromResult = {
        #Ok : Icrc1BlockIndex;
        #Err : TransferFromError;
    };

    public type CanisterId = Principal;

    public type BlockHeight = Nat64;

    public type TransactionNotification = {
        from: Principal;
        from_subaccount: ?SubAccount;
        to: CanisterId;
        to_subaccount: ?SubAccount;
        block_height: BlockHeight;
        amount: ICP;
        memo: Memo;
    };

    public type Ledger = actor {
        transfer : shared (TransferArgs) -> async TransferResult;

        icrc2_transfer_from : shared (TransferFromArgs) -> async (TransferFromResult);

        icrc1_balance_of : shared query (Account) -> async (Icrc1Tokens);
    };

};