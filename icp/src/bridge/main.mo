import Principal "mo:base/Principal";
import Text "mo:base/Text";
import HashMap "mo:base/HashMap";
import Array "mo:base/Array";
import Bool "mo:base/Bool";
import Nat "mo:base/Nat";
import Option "mo:base/Option";
import Error "mo:base/Error";
import Nat32 "mo:base/Nat32";
import Prelude "mo:base/Prelude";
import Blob "mo:base/Blob";
import Iter "mo:base/Iter";
import Nat64 "mo:base/Nat64";
import ExperimentalCycles "mo:base/ExperimentalCycles";
import Debug "mo:base/Debug";
import Hash "mo:base/Hash";
import Map "mo:map/Map";
import { thash } "mo:map/Map";
import CollectionFactory "../collection-factory/main";
import StorageFactory "../storage-factory/main";
import Lib "mo:ed25519/lib";
import DuplicateToOriginalMappingKey "./structures/duplicate_to_original_mapping";
import OriginalToDuplicateMappingKey "./structures/original_to_duplicate_mapping";
import Service "mo:icrc37-mo/service";
import NFT "../nft/main";
import Storage "../storage/main";
import Validator "structures/validator";
import LockedEvent "structures/locked_event";
import SignerAndSignature "structures/signer_and_signature";
import ClaimData "structures/claim_data";
import ClaimedEvent "structures/claimed_event";
import AddValidator "structures/add_validator";
import BlacklistValidator "structures/blacklist_validator";
import Types "types";
actor class XPBridge(
  _args : {
    validators : [(Text, Principal)];
    chain_type : Text;
    collection_deployer : Principal;
    storage_deployer : Principal;
  }
) = self {

  let Ledger : Types.Ledger = actor("ryjl3-tyaaa-aaaaa-aaaba-cai");

  type OriginalToDuplicateMappingKey = OriginalToDuplicateMappingKey.OriginalToDuplicateMappingKey;
  type DuplicateToOriginalMappingKey = DuplicateToOriginalMappingKey.DuplicateToOriginalMappingKey;
  type StorageMappingKey = OriginalToDuplicateMappingKey.OriginalToDuplicateMappingKey;

  type Validator = Validator.Validator;
  type LockedEvent = LockedEvent.LockedEvent;
  type ClaimedEvent = ClaimedEvent.ClaimedEvent;
  type SignerAndSignature = SignerAndSignature.SignerAndSignature;
  type ClaimData = ClaimData.ClaimData;

  private func validatorsArrayToHashMap(vals : [(Text, Principal)]) : HashMap.HashMap<Text, Validator> {
    var tup = Array.map<(Text, Principal), (Text, Validator)>(vals, func x = (x.0, { address = x.1; pending_rewards = 0 }));
    HashMap.fromIter(tup.vals(), tup.size(), Text.equal, Text.hash);
  };
  type PrincipalPrincipalTuple = (Text, Text);
  private stable var self_chain = _args.chain_type;
  private stable var validators_count = _args.validators.size();
  private stable var singular = "singular";
  private stable var _multiple = "multiple";
  private var validators : HashMap.HashMap<Text, Validator> = validatorsArrayToHashMap(_args.validators);
  private var blacklisted_validators: HashMap.HashMap<Text, Bool> = HashMap.fromIter([].vals(), 0, Text.equal, Text.hash);
  private var original_to_duplicate_mapping = HashMap.fromIter<OriginalToDuplicateMappingKey, Principal>([].vals(), 0, OriginalToDuplicateMappingKey.equal, OriginalToDuplicateMappingKey.hash);
  private var duplicate_to_original_mapping = HashMap.fromIter<DuplicateToOriginalMappingKey, Principal>([].vals(), 0, DuplicateToOriginalMappingKey.equal, DuplicateToOriginalMappingKey.hash);
  private var unique_identifiers = HashMap.fromIter<Text, Bool>([].vals(), 0, Text.equal, Text.hash);
  private stable var collection_factory = actor (Principal.toText(_args.collection_deployer)) : CollectionFactory.CollectionFactory;
  private stable var storage_factory = actor (Principal.toText(_args.storage_deployer)) : StorageFactory.StorageFactory;

  private var original_storage = HashMap.fromIter<StorageMappingKey, Principal>([].vals(), 0, OriginalToDuplicateMappingKey.equal, OriginalToDuplicateMappingKey.hash);
  private var duplicate_storage = HashMap.fromIter<StorageMappingKey, Principal>([].vals(), 0, OriginalToDuplicateMappingKey.equal, OriginalToDuplicateMappingKey.hash);
  private var locked_events = HashMap.fromIter<Text, LockedEvent>([].vals(), 0, Text.equal, Text.hash);
  private var claimed_events = HashMap.fromIter<Text, ClaimedEvent>([].vals(), 0, Text.equal, Text.hash);
  private stable var nonce = 0;
  private stable var claim_nonce = 0;
  private var nonce_to_hash = HashMap.fromIter<Nat, Text>([].vals(), 0, Nat.equal, Hash.hash);
  private var nonce_to_claim_hash = HashMap.fromIter<Nat, Text>([].vals(), 0, Nat.equal, Hash.hash);
  
  private stable var _init = false;

  public shared func init() : async () {
    if (_init == false) {
      await collection_factory.set_owner(Principal.fromActor(self));
      await storage_factory.set_owner(Principal.fromActor(self));
    } else {
      _init := true;
    };
  };
  private func verify_signatures(hash : [Nat8], signers_and_signatures : [SignerAndSignature]) : (Nat, [Text]) {
    var percent = 0;
    let has_signer = Map.new<Text, Bool>();
    label signatures for (ss in signers_and_signatures.vals()) {
      Debug.print(debug_show ss);
      if (Option.isSome(validators.get(ss.signer))) {
        if (not Map.has(has_signer, thash, ss.signer)) {
          let ok = Lib.ED25519.verify(Lib.Utils.hexToBytes(ss.signature), hash, Lib.Utils.hexToBytes(ss.signer));
          Debug.print("Is OK: " # debug_show ok);
          if (ok) {
            Map.set(has_signer, thash, ss.signer, true);
            percent := percent + 1;
          };
        };
      } else {
        continue signatures;
      };
    };
    (percent, Iter.toArray(Map.keys(has_signer)));
  };

  private func required_threshold() : Nat {
    return ((validators_count * 2) / 3) + 1;
  };

  public func add_validator(add_validator: AddValidator.AddValidator, sigs : [SignerAndSignature]) : async () {
    let pubk = add_validator.public_key;
    let princ = add_validator.principal;

    let is_blacklisted = Option.isSome(blacklisted_validators.get(pubk));
    if (is_blacklisted) {
      throw Error.reject("Validator is blacklisted.");
    };

    let present = Option.isSome(validators.get(pubk));
    if (present) {
      throw Error.reject("Validator already present.");
    };
    let ahash = AddValidator.hash(add_validator);
    let (percent, _) = verify_signatures(Blob.toArray(ahash), sigs);
    if (percent < required_threshold()) {
      throw Error.reject("Threshold not reached.");
    };
    validators.put(
      pubk,
      {
        address = princ;
        pending_rewards = 0;
      },
    );
    validators_count += 1;
  };

  public func blacklist_validator(bv: BlacklistValidator.BlacklistValidator, sigs : [SignerAndSignature]): async () {
    let pubk = bv.public_key;
    let present = Option.isSome(validators.get(pubk));
    if (not present) {
      throw Error.reject("Validator is not added");
    };
    let bhash = BlacklistValidator.hash(bv);
    let (percent, _) = verify_signatures(Blob.toArray(bhash), sigs);
    if (percent < required_threshold()) {
      throw Error.reject("Threshold not reached.");
    };
    validators.delete(pubk);
    validators_count -= 1;
    blacklisted_validators.put(pubk, true);
  };

  public shared (msg) func lock_nft(source_nft_contract_address : Principal, tid : Nat, destination_chain : Text, destination_user_address : Text, metadata_uri : Text) : async Text {
    if (destination_chain == self_chain) {
      throw Error.reject("Destination chain cannot be equal to self chain");
    };
    let original_collection_address = duplicate_to_original_mapping.get({
      source_nft_contract_address = source_nft_contract_address;
      source_chain = self_chain;
    });
    let is_original = Option.isNull(original_collection_address);
    if (is_original) {
      await transfer_to_storage(original_storage, source_nft_contract_address, tid, msg.caller);
      let locked : LockedEvent = {
        source_nft_contract_address = source_nft_contract_address;
        source_chain = self_chain;
        destination_chain = destination_chain;
        destination_user_address = destination_user_address;
        nft_type = singular;
        token_amount = 1;
        token_id = tid;
        metadata_uri = metadata_uri;
        sender_address = Principal.toText(msg.caller);
      };
      let hash = Nat32.toText(LockedEvent.hash(locked));
      locked_events.put(hash, locked);
      nonce_to_hash.put(nonce, hash);
      nonce+=1;
      return hash;
    } else {
      await transfer_to_storage(duplicate_storage, source_nft_contract_address, tid, msg.caller);
      let locked : LockedEvent = {
        source_nft_contract_address = source_nft_contract_address;
        source_chain = self_chain;
        destination_chain = destination_chain;
        destination_user_address = destination_user_address;
        nft_type = singular;
        token_amount = 1;
        token_id = tid;
        metadata_uri = metadata_uri;
        sender_address = Principal.toText(msg.caller);
      };
      let hash = Nat32.toText(LockedEvent.hash(locked));
      locked_events.put(hash, locked);
      nonce_to_hash.put(nonce, hash);
      nonce+=1;
      return hash;
    };
  };

  private func transfer_to_storage(mapping : HashMap.HashMap<StorageMappingKey, Principal>, source_nft_contract_address : Principal, tid : Nat, sender : Principal) : async () {
    let src = Principal.toText(source_nft_contract_address);
    let sm = mapping.get({
      source_nft_contract_address = src;
      source_chain = self_chain;
    });

    switch sm {
      case (null) {
        let storage_contract = await storage_factory.deploy_storage(src);
        let nft = actor (Principal.toText(source_nft_contract_address)) : Service.Service;
        let result = (await nft.icrc37_transfer_from([{ from = { owner = sender; subaccount = null }; to = { owner = storage_contract; subaccount = null }; created_at_time = null; token_id = tid; spender_subaccount = null; memo = null }]))[0];
        switch (result) {
          case (?value) {
            switch (value) {
              case (#Ok(_)) {};
              case (#Err(_)) {
                throw Error.reject("Storage: Failed to lock token" # debug_show value);
              };
            };
          };
          case (null) {
            throw Error.reject("Storage: Got no response for transfer call");
          };
        };
      };
      case (?storage_contract) {
        let nft = actor (Principal.toText(source_nft_contract_address)) : Service.Service;
        let result = (await nft.icrc37_transfer_from([{ from = { owner = sender; subaccount = null }; to = { owner = storage_contract; subaccount = null }; created_at_time = null; token_id = tid; spender_subaccount = null; memo = null }]))[0];
        switch (result) {
          case (?value) {
            switch (value) {
              case (#Ok(_)) {};
              case (#Err(e)) {
                throw Error.reject("Storage: Failed to lock token" # debug_show e);
              };
            };
          };
          case (null) {
            throw Error.reject("Storage: Got no response for transfer call");
          };
        };
      };
    };
  };

  public shared ({ caller }) func claim_nft(claim_data : ClaimData, sigs : [SignerAndSignature]) : async Text {
    if (claim_data.nft_type != singular) {
      throw Error.reject("Invalid NFT type!");
    };
    try {
      let transferResult = await Ledger.icrc2_transfer_from({
        from = {
          owner = caller;
          subaccount = null;
        };
        memo = null;
        amount = Nat64.toNat(claim_data.fee);
        fee = null;
        from_subaccount = null;
        to = {
          owner = Principal.fromActor(self);
          subaccount = null;
        };
        created_at_time = null;
        spender_subaccount = null;
      });
      // check if the transfer was successfull
      switch (transferResult) {
        case (#Err(transferError)) {
          throw Error.reject("Couldn't transfer funds:\n" # debug_show (transferError));
        };
        case (_) {};
      };
    } catch (e) {
      throw Error.reject("Failed to transfer tokens. Reject message: " # Error.message(e));
    };
    let chash = ClaimData.hash(claim_data);
    let mhash = Blob.hash(chash);
    let hash = Nat32.toText(mhash);

    if (unique_identifiers.get(hash) != null) {
      throw Error.reject("Data already processed!");
    };
    let (percent, validators) = verify_signatures(Blob.toArray(chash), sigs);
    if (percent < required_threshold()) {
      throw Error.reject("Threshold not reached.");
    };
    await reward_validators(claim_data.fee, validators);
    let duplicate_collection_address = original_to_duplicate_mapping.get({
      source_chain = claim_data.source_chain;
      source_nft_contract_address = claim_data.source_nft_contract_address;
    });
    let has_duplicate = Option.isSome(duplicate_collection_address);
    let storage = switch (duplicate_collection_address) {
      case (null) {
        original_storage.get({
          source_nft_contract_address = claim_data.source_nft_contract_address;
          source_chain = claim_data.source_chain;
        });
      };
      case (?dca) {
        duplicate_storage.get({
          source_chain = self_chain;
          source_nft_contract_address = Principal.toText(dca);
        });
      };
    };
    let has_storage = Option.isSome(storage);
    if (has_duplicate and has_storage) {
      let sc = o_unwrap(storage);
      let collection = actor (Principal.toText(o_unwrap(duplicate_collection_address))) : NFT.NFT;
      let owner = (await collection.icrc7_owner_of([claim_data.token_id]))[0];

      if (o_unwrap(owner).owner == sc) {
        let _unlock = await unlock_nft(sc, claim_data.token_id, claim_data.destination_user_address);
      } else {
        let _mint = collection.icrcX_mint(claim_data.token_id, { owner = claim_data.destination_user_address; subaccount = null }, claim_data.metadata);
      };
      unique_identifiers.put(hash, true);

      let claim_hash = emit_claimed_event(claim_data.lock_tx_chain, claim_data.source_chain, Principal.toText(o_unwrap(duplicate_collection_address)), claim_data.token_id, claim_data.transaction_hash);
      nonce_to_claim_hash.put(claim_nonce, claim_hash);
      claim_nonce+=1;

      return claim_hash;
    
    } else if (has_duplicate and not has_storage) {
      let collection = actor (Principal.toText(o_unwrap(duplicate_collection_address))) : NFT.NFT;
      let _mint = await collection.icrcX_mint(claim_data.token_id, { owner = claim_data.destination_user_address; subaccount = null }, claim_data.metadata);
      unique_identifiers.put(hash, true);
      let claim_hash = emit_claimed_event(claim_data.lock_tx_chain, claim_data.source_chain, Principal.toText(o_unwrap(duplicate_collection_address)), claim_data.token_id, claim_data.transaction_hash);
      nonce_to_claim_hash.put(claim_nonce, claim_hash);
      claim_nonce+=1;

      return claim_hash;
      // Emit Claimed EV;
    } else if (not has_duplicate and not has_storage) {
      let new_collection_address = await collection_factory.deploy_nft_collection(claim_data.name, claim_data.symbol);
      original_to_duplicate_mapping.put(
        {
          source_chain = claim_data.source_chain;
          source_nft_contract_address = claim_data.source_nft_contract_address;
        },
        new_collection_address,
      );
      let collection = actor (Principal.toText(new_collection_address)) : NFT.NFT;
      let _mint = await collection.icrcX_mint(claim_data.token_id, { owner = claim_data.destination_user_address; subaccount = null }, claim_data.metadata);
      unique_identifiers.put(hash, true);
      let claim_hash = emit_claimed_event(claim_data.lock_tx_chain, claim_data.source_chain, Principal.toText(new_collection_address), claim_data.token_id, claim_data.transaction_hash);
      nonce_to_claim_hash.put(claim_nonce, claim_hash);
      claim_nonce+=1;

      return claim_hash;
    
    } else if (not has_duplicate and has_storage) {
      let sc = o_unwrap(storage);
      let collection = actor (claim_data.source_nft_contract_address) : NFT.NFT;
      let owner = (await collection.icrc7_owner_of([claim_data.token_id]))[0];
      if (o_unwrap(owner).owner == sc) {
        let _unlock = await unlock_nft(sc, claim_data.token_id, claim_data.destination_user_address);
      } else {
        let _mint = collection.icrcX_mint(claim_data.token_id, { owner = claim_data.destination_user_address; subaccount = null }, claim_data.metadata);
      };
      unique_identifiers.put(hash, true);
      let claim_hash = emit_claimed_event(claim_data.lock_tx_chain, claim_data.source_chain, claim_data.source_nft_contract_address, claim_data.token_id, claim_data.transaction_hash);
      nonce_to_claim_hash.put(claim_nonce, claim_hash);
      claim_nonce+=1;

      return claim_hash;

    } else {
      Prelude.unreachable();
    };
  };

  public func claim_validator_rewards(publicKey : Text) : async (Nat64, Nat64) {
    switch (validators.get(publicKey)) {
      case (null) {
        throw Error.reject("No Such Validator Found.");
      };
      case (?v) {
        let pr = v.pending_rewards;
        try {
          let result = await Ledger.transfer({
            from = Principal.fromActor(self);
            created_at_time = null;
            to = Principal.toLedgerAccount(v.address, null);
            memo = 0;
            fee = { e8s = 10_000 };
            amount = { e8s = pr };
            from_subaccount = null;
          });
          switch (result) {
            case (#Err(transferError)) {
              throw Error.reject("Couldn't transfer funds:\n" # debug_show (transferError));
            };
            case (#Ok(e)) {
              validators.put(publicKey, { pending_rewards = 0; address = v.address });
              return (e, pr);
            };
          };
        } catch (e) {
          throw Error.reject("Failed to transfer ICP to Validator." # Error.message(e));
        };
      };
    };
  };

  private func reward_validators(fee : Nat64, vals : [Text]) : async () {
    let me = Principal.fromActor(self);
    let total_rewards = await Ledger.icrc1_balance_of({
      owner = me;
      subaccount = null;
    });
    let fee_per_validator = total_rewards / validators.size();
    assert Nat64.fromNat(total_rewards) >= fee;
    for (v in vals.vals()) {
      let validator = o_unwrap(validators.get(v));
      let prev = validator.pending_rewards;
      validators.put(v, { address = validator.address; pending_rewards = prev + Nat64.fromNat(fee_per_validator) });
    };
    return;
  };

  private func emit_claimed_event(lock_tx_chain: Text, source_chain : Text, nft_contract : Text, token_id : Nat, transaction_hash : Text) : Text {
    let claimed = {
      source_chain = source_chain;
      nft_contract = Principal.fromText(nft_contract);
      token_id = token_id;
      transaction_hash = transaction_hash;
      lock_tx_chain = lock_tx_chain;
    };
    let hash = Nat32.toText(ClaimedEvent.hash(claimed));
    claimed_events.put(hash, claimed);
    return hash;
  };

  private func unlock_nft(storage : Principal, tid : Nat, destination_user_address : Principal) : async () {
    let s = actor (Principal.toText(storage)) : Storage.Storage;
    await s.unlock_token(tid, { owner = destination_user_address; subaccount = null });
  };

  private func o_unwrap<T>(o : ?T) : T {
    switch o {
      case (null) {
        Prelude.unreachable();
      };
      case (?value) {
        return value;
      };
    };
  };

  public query func get_locked_data(hash: Text): async ?LockedEvent {
    return locked_events.get(hash);
  };

  public query func get_claimed_data(hash: Text): async ?ClaimedEvent {
    return claimed_events.get(hash);
  };

  public query func get_blacklisted_validators(pubk: Text): async ?Bool {
    return blacklisted_validators.get(pubk);
  };

  public query func get_validator(pubk: Text): async ?Validator {
    return validators.get(pubk);
  };

  public query func encode_claim_data(claim_data: ClaimData): async Blob {
    return ClaimData.hash(claim_data);
  };

   public query func encode_add_validator(av: AddValidator.AddValidator): async Blob {
    return AddValidator.hash(av);
  };

  public query func encode_blacklist_validator(bv: BlacklistValidator.BlacklistValidator): async Blob {
    return BlacklistValidator.hash(bv);
  };

  public query func get_nonce(): async Nat {
    return nonce;
  };

  public query func get_claim_nonce(): async Nat {
    return claim_nonce;
  };

  public query func get_hash_from_nonce(nonce: Nat): async ?Text {
    return nonce_to_hash.get(nonce);
  };

  public query func get_hash_from_claim_nonce(nonce: Nat): async ?Text {
    return nonce_to_claim_hash.get(nonce);
  };

  public query func get_validator_count(): async Nat {
    return validators_count;
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
