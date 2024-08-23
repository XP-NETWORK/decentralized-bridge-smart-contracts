import { expect, it, test, vitest } from "vitest";
import {
  Actor,
  CanisterStatus,
  Ed25519PublicKey,
  HttpAgent,
} from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { bridge, ledger, nft } from "./actor";
import { ClaimData } from "../declarations/bridge/bridge.did";
import { IDL } from "@dfinity/candid";
import * as ed from "@noble/ed25519";
import { Ed25519KeyIdentity } from "@dfinity/identity";

const pk = Buffer.from(
  "efeb1c2fde35ba8cada52300388a0455f5f75e20bd15efdf3a2b29af172a3379",
  "hex"
);
const publicKey = Buffer.from(
  "26e4437872140b2d52e788dd8ba135ffc8f24fbfe5aede6c95d5f2516803b234",
  "hex"
);

const owner = Principal.fromText(
  "wwwmd-yesc2-u4piz-sf6dp-yqcxd-fkyit-35i6q-h2ztg-rvfhf-7tand-cae"
);

async function mint_nft() {
  const toMint = Math.floor(Math.random() * 10000);
  const mint = await nft.icrcX_mint(
    BigInt(toMint),
    { owner: owner, subaccount: [] },
    "metadata"
  );
  console.log(mint)
  return BigInt(toMint);
}

async function approve_nft(token_id: bigint) {
  const [approval] = await nft.icrc37_approve_tokens([
    {
      approval_info: {
        created_at_time: [],
        spender: {
          owner: Actor.canisterIdOf(bridge),
          subaccount: [],
        },
        expires_at: [],
        from_subaccount: [],
        memo: [],
      },
      token_id,
    },
  ]);
  return token_id;
}

test(
  "should be able to lock nft",
  {
    timeout: 500000000,
  },
  async () => {
    const respones = await mint_nft();
    const approve = await approve_nft(respones);
    const lock = await bridge.lock_nft(
      Actor.canisterIdOf(nft),
      respones,
      "BSC",
      "0x.."
    );
    const [locked_event] = await bridge.get_locked_data(lock);
    expect(locked_event).toBeDefined();
    expect(locked_event?.destination_chain).toBe("BSC");
    expect(locked_event?.destination_user_address).toBe("0x..");
    expect(locked_event?.source_nft_contract_address.toString()).toBe(
      Actor.canisterIdOf(nft).toString()
    );
    expect(locked_event?.token_id.toString()).toBe(approve.toString());
  },
);

test("should be able to add validator", async () => {
  const newPk = ed.utils.randomPrivateKey();
  const newPublicKey = await ed.getPublicKey(newPk);
  const principal = Ed25519KeyIdentity.fromSecretKey(newPk).getPrincipal();
  const ava = {
    public_key: Buffer.from(newPublicKey).toString("hex"),
    principal: principal,
  };
  const vc = await bridge.get_validator_count()
  const AddValidator = IDL.Record({
    principal: IDL.Principal,
    public_key: IDL.Text,
  });
  const enc = AddValidator.encodeValue(ava);
  const encode = await bridge.encode_add_validator(ava)
  const signature = await ed.sign(Buffer.from(encode), pk);
  const add = await bridge.add_validator(ava, [
    {
      signature: Buffer.from(signature).toString("hex"),
      signer: publicKey.toString("hex"),
    },
  ]);
  expect(await bridge.get_validator_count()).toBe(vc + 1n);
});

test(
  "should be able to claim nft",
  {
    timeout: Infinity,
  },
  async () => {
    const balance = await ledger.icrc1_balance_of({
      owner: Principal.fromText(
        "wwwmd-yesc2-u4piz-sf6dp-yqcxd-fkyit-35i6q-h2ztg-rvfhf-7tand-cae"
      ),
      subaccount: [],
    });
    console.log(`Balance: ${balance}`);
    const approved = await ledger.icrc2_approve({
      amount: 1000000n,
      created_at_time: [],
      expected_allowance: [],
      expires_at: [],
      fee: [],
      from_subaccount: [],
      memo: [],
      spender: {
        owner: Actor.canisterIdOf(bridge),
        subaccount: [],
      },
    });
    const cd: ClaimData = {
      destination_chain: "ICP",
      destination_user_address: Principal.fromText("2vxsx-fae"),
      fee: 500000n,
      metadata: "metadata",
      name: "NewName",
      nft_type: "singular",
      royalty: 1n,
      royalty_receiver: Principal.fromText("2vxsx-fae"),
      source_chain: "BSC",
      source_nft_contract_address: "0x...",
      symbol: "NNBSC",
      token_amount: 1n,
      token_id: BigInt(Math.floor(Math.random() * 10000)),
      transaction_hash: "0x0000",
      lock_tx_chain: "nigga",
    };
    const encoded = await bridge.encode_claim_data(cd)
    const signature = await ed.sign(Buffer.from(encoded), pk);

    const claim = await bridge.claim_nft(cd, [
      {
        signature: Buffer.from(signature).toString("hex"),
        signer: publicKey.toString("hex"),
      }, 
    ]);

    console.log(claim);
  },
  
);


test("should be able to withdraw rewards", async () => {
  const response = await bridge.claim_validator_rewards(publicKey.toString("hex"));
  console.log(response)
});
