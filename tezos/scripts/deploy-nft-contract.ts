import { InMemorySigner } from "@taquito/signer";
import {
  BigMapAbstraction,
  MichelsonMap,
  TezosToolkit,
} from "@taquito/taquito";
import { config } from "dotenv";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import NftContract from "../build/NFT.json";
import { NFTContractType } from "../types/NFT.types";
import { tas } from "../types/type-aliases";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;


export async function deployNftStorage() {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);
  const signer = await InMemorySigner.fromSecretKey(SK);
  Tezos.setProvider({
    signer: signer,
  });

  try {
    const originated = await Tezos.contract.originate<NFTContractType>({
      code: NftContract,
      storage: {
        approvals: tas.bigMap([]),
        assets: tas.bigMap([]),
        extension:  tas.address(await signer.publicKeyHash()),
        metadata: tas.bigMap([]),
        operators: null,
        proxy: tas.address(await signer.publicKeyHash()),
        token_metadata: tas.bigMap([]),
      },
    });
    logToConsole(
      `Waiting for NFT Contract: ${originated.contractAddress} to be confirmed...`
    );
    await originated.confirmation(2);
    logToConsole("NFT: ", originated.contractAddress);
    return originated.contractAddress!;
  } catch (error: any) {
    logToConsole(error);
    return undefined;
  }
}

if (require.main === module) {
  (async () => {
    // const initialStorage = {
    //   owner: await stdio.question("Enter owner address: "),
    //   collection: await stdio.question("Enter the collection (FA2) address: "),
    // };
    deployNftStorage();
  })();
}

export function logToConsole(...args: any[]) {
  if (require.main === module) {
    console.log(args);
  }
}
