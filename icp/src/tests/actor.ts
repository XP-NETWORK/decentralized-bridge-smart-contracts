import { Actor, HttpAgent, HttpAgentOptions } from "@dfinity/agent";
//@ts-ignore no types cope
import fetch from "isomorphic-fetch";
import canisterIds from ".dfx/local/canister_ids.json";
import {
  _SERVICE as BridgeService,
  idlFactory as BridgeIDL,
} from "../declarations/bridge/bridge.did.js";
import {
  _SERVICE as NftService,
  idlFactory as NftIDL,
} from "../declarations/nft/nft.did.js";
import { Ed25519KeyIdentity } from "@dfinity/identity";

export const createActor = async <T>(
  canisterId: string,
  idlFactory: any,
  options: HttpAgentOptions
) => {
    const identity = Ed25519KeyIdentity.fromSecretKey(Buffer.from("efeb1c2fde35ba8cada52300388a0455f5f75e20bd15efdf3a2b29af172a3379", "hex"))
  const agent = await HttpAgent.create({ ...options, shouldFetchRootKey: true, identity });
  const rk = await agent.fetchRootKey();


  // Creates an actor with using the candid interface and the HttpAgent
  return Actor.createActor<T>(idlFactory, {
    agent,
    canisterId,
    ...options,
  });
};
console.log(canisterIds)

export const bridgeCanister = canisterIds.bridge.local;
// export const nftCanister = canisterIds.nft.local;

export const bridge = await createActor<BridgeService>(bridgeCanister, BridgeIDL, {
  host: "http://127.0.0.1:4943",
  fetch,
});

// export const nft = await createActor<NftService>(nftCanister, NftIDL, {
//   host: "http://127.0.0.1:4943",
//   fetch,
// });
