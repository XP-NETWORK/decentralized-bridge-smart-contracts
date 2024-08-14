import { InMemorySigner } from "@taquito/signer";
import { MichelsonMap, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import NFTCollectionFactory from "../build/NFTCollectionFactory.json"
import SFTCollectionFactory from "../build/SFTCollectionFactory.json"

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function deployCollectionFactory(owner: string | undefined) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });

  try {
    const nftC = await Tezos.contract.originate({
      code: NFTCollectionFactory,
      storage: {
        owner: owner,
        collection_to_store: new MichelsonMap(),
      },
    });
     console.log(
      `Waiting for NFT Collection Factory Contract ${nftC.contractAddress} to be confirmed...`
    );
    await nftC.confirmation(2);
     const sftC = await Tezos.contract.originate({
      code: SFTCollectionFactory,
      storage: {
        owner: owner,
        collection_to_store: new MichelsonMap(),
      },
    });
     console.log(
      `Waiting for SFT Collection Factory Contract ${sftC.contractAddress} to be confirmed...`
    );
    await sftC.confirmation(2);
    return [nftC.contractAddress!, sftC.contractAddress!];
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
