import { expect, it, test, vitest } from "vitest";
import { Actor, CanisterStatus, Ed25519PublicKey, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { bridge, nft } from "./actor";
import { ClaimData } from "../declarations/bridge/bridge.did";
import { IDL } from "@dfinity/candid";
import * as ed from "@noble/ed25519"

const pk = Buffer.from(
  "efeb1c2fde35ba8cada52300388a0455f5f75e20bd15efdf3a2b29af172a3379",
  "hex"
);
const publicKey = Buffer.from(
  "d0b8d423400e58f9aa0d2a1dcbe55aa5e079bf08b58c150c8a6bd99def371bb3",
  "hex"
);

async function mint_nft() {
  const toMint = Math.floor(Math.random() * 10000);
  const mint = await nft.icrcX_mint(
    BigInt(toMint),
    { owner: Principal.fromText("2vxsx-fae"), subaccount: [] },
    "metadata"
  );
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

test("should be able to lock nft", async () => {
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
}, {
  timeout: 500000000
});

test("should be able to claim nft", async () => {
  const cd: ClaimData = {
    destination_chain: "ICP",
    destination_user_address: Principal.fromText("2vxsx-fae"),
    fee: 500000n,
    metadata: "metadata",
    name: "NewName",
    nft_type: "singular",
    royalty: 10n,
    royalty_receiver: Principal.fromText("2vxsx-fae"),
    source_chain: "BSC",
    source_nft_contract_address: "0x...",
    symbol: "NNBSC",
    token_amount: 1n,
    token_id: 25n,
    transaction_hash: '0x00'
  };
   const ClaimData = IDL.Record({
     fee: IDL.Nat64,
     source_chain: IDL.Text,
     transaction_hash: IDL.Text,
     token_amount: IDL.Nat,
     destination_chain: IDL.Text,
     token_id: IDL.Nat,
     source_nft_contract_address: IDL.Text,
     metadata: IDL.Text,
     name: IDL.Text,
     nft_type: IDL.Text,
     royalty: IDL.Nat,
     royalty_receiver: IDL.Principal,
     destination_user_address: IDL.Principal,
     symbol: IDL.Text,
   });
   const encoded = ClaimData.encodeValue(cd);
   const signature = await ed.sign(Buffer.from(encoded), pk);

   const claim = await bridge.claim_nft(cd, [
     {
       signature: Buffer.from(signature).toString("hex"),
       signer: publicKey.toString("hex")
     },
   ]);

   console.log(claim)
});
