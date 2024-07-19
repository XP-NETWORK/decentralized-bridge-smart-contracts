import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";
import { BigNumber } from "bignumber.js";
import { config } from "dotenv";
import { tas } from "../types/type-aliases";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import { NFTContractType } from "../types/NFT.types";
import { Tzip16Module, tzip16 } from "@taquito/tzip16";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function get_token_metadata(token_id: bigint, contract: string) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });
  const nft = await Tezos.contract.at<NFTContractType>(contract);
  const token_md = await (
    await nft.storage()
  ).token_metadata.get(tas.nat(token_id.toString()));
  const md = token_md.token_info.keys().next();
  return md.value;
}
export async function get_contract_metadata(contract: string) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);
  Tezos.addExtension(new Tzip16Module());

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });
  const nft = await Tezos.contract.at(contract, tzip16);
  const metadata = await nft.tzip16();
  return metadata;
}

if (require.main === module) {
  (async () => {
    const tid = await stdio.question("Enter token id: ");
    const contract = await stdio.question("Enter contract address: ");

    await get_token_metadata(BigInt(tid), contract);
  })();
}
