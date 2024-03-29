import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";

import { config } from "dotenv";
import { tas } from "../types/type-aliases";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import { NFTContractType } from "../types/NFT.types";
import { BridgeContractType } from "../types/Bridge.types";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function get_token_metadata(contract: string) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });
  const bridge = await Tezos.contract.at<BridgeContractType>(contract);
  const storage = await bridge.storage();
  console.log(storage.validators_count.toString());
  console.log(
    await storage.validators.get(
      tas.address("tz1hmsQEAzt7F1y7X6xjv1U4pqk4xeKKKPcR")
    )
  );
}

if (require.main === module) {
  (async () => {
    const contract = await stdio.question("Enter contract address: ");

    await get_token_metadata(contract);
  })();
}
