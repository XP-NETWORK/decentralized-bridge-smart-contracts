import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

export type NFTStorageInitArgs = { owner: string; collection: string };

import SftContract from "../artifacts/SFTStorage.json";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function deploySftStorage(storage: NFTStorageInitArgs) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });

  try {
    const originated = await Tezos.contract.originate({
      code: SftContract,
      storage: storage,
    });
    logToConsole(
      `Waiting for SFT Storage Contract ${originated.contractAddress} to be confirmed...`
    );
    await originated.confirmation(2);
    logToConsole("SFT Storage Contract: ", originated.contractAddress);
    return originated.contractAddress!;
  } catch (error: any) {
    logToConsole(error);
    return undefined;
  }
}

if (require.main === module) {
  (async () => {
    const storage = {
      owner: await stdio.question("Enter owner address: "),
      collection: await stdio.question("Enter the collection (FA2) address: "),
    };
    deploySftStorage(storage);
  })();
}

export function logToConsole(...args: any[]) {
  if (require.main === module) {
    console.log(args);
  }
}
