import { InMemorySigner } from "@taquito/signer";
import {
  Context,
  LegacyWalletProvider,
  MichelCodecPacker,
  TezosToolkit,
  Wallet,
} from "@taquito/taquito";

import { config } from "dotenv";
import { tas } from "../types/type-aliases";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import { BridgeContractType } from "../types/Bridge.types";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function get_token_metadata(contract: string) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  const signer = await InMemorySigner.fromSecretKey(SK);

  Tezos.setProvider({
    signer,
  });
  console.log(await Tezos.signer.publicKeyHash());
  const packer = new MichelCodecPacker();
  const packed = await packer.packData({
    data: {
      string: await Tezos.signer.publicKeyHash(),
    },
    type: {
      prim: "address",
    },
  });
  console.log(packed);

  const bridge = await Tezos.contract.at<BridgeContractType>(contract);
  const signature = await signer.sign(packed.packed);
  const tx = await bridge.methods
    .add_validator(tas.address(await Tezos.signer.publicKeyHash()), [
      {
        addr: tas.address(await Tezos.signer.publicKeyHash()),
        sig: tas.signature(signature.sig),
        signer: tas.key(await signer.publicKey()),
      },
    ])
    .send();
  console.log(tx);
}

if (require.main === module) {
  (async () => {
    const contract = await stdio.question("Enter contract address: ");

    await get_token_metadata(contract);
  })();
}
