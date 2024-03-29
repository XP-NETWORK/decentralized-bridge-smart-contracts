import { InMemorySigner } from "@taquito/signer";
import { MichelsonMap, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import CollectionFactory from "../artifacts/CollectionFactory.json";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function deployCollectionFactory(owner: string | undefined) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });

  try {
    const originated = await Tezos.contract.originate({
      code: CollectionFactory,
      storage: {
        owner: owner,
        collection_to_store: new MichelsonMap(),
      },
    });
    console.log(
      `Waiting for Collection Factory Contract ${originated.contractAddress} to be confirmed...`
    );
    await originated.confirmation(2);
    console.log("Collection Factory Contract: ", originated.contractAddress);
    return originated.contractAddress;
  } catch (error: any) {
    console.log(error);
    return undefined;
  }
}

if (require.main === module) {
  (async () => {
    const ownerStr = await stdio.question(
      "Enter owner address (or leave empty): "
    );
    const owner = ownerStr !== "" ? ownerStr : undefined;

    await deployCollectionFactory(owner);
  })();
}
