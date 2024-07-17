import { expect, test } from "vitest";
import { Actor, CanisterStatus, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { bridge, bridgeCanister, nft, nftCanister } from "./actor";
import {utils, getPublicKey} from "@noble/ed25519"


const pk = Buffer.from(
  "efeb1c2fde35ba8cada52300388a0455f5f75e20bd15efdf3a2b29af172a3379", 'hex'
);
const publicKey = Buffer.from(
  "d0b8d423400e58f9aa0d2a1dcbe55aa5e079bf08b58c150c8a6bd99def371bb3", 'hex'
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

async function approve_nft(token_id:bigint) {
    const [approval] = await nft.icrc37_approve_tokens([{
        approval_info: {
            created_at_time: [],
            spender: {
                owner: Actor.canisterIdOf(bridge),
                subaccount: []
            },
            expires_at: [],
            from_subaccount: [],
            memo: []
        },
        token_id
    }])
    return token_id
}

test("should handle a basic greeting", async () => {
  const respones = await mint_nft();
  const approve = await approve_nft(respones);
  const lock = await bridge.lock_nft(Actor.canisterIdOf(nft), approve, "BSC", "0x..");
  const [locked_event] = await bridge.get_locked_data(lock)
  expect(locked_event).toBeDefined();
  expect(locked_event?.destination_chain).toBe("BSC")
  expect(locked_event?.destination_user_address).toBe("0x..");
  expect(locked_event?.source_nft_contract_address.toString()).toBe(
    Actor.canisterIdOf(nft).toString()
  );
    expect(locked_event?.token_id.toString()).toBe(
      approve.toString()
    );
});
