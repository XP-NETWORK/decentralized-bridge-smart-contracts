import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";
import { Schema } from "@taquito/michelson-encoder";
import { packDataBytes, MichelsonType } from "@taquito/michel-codec";

import { keccak256 } from "@ethersproject/keccak256";

import { config } from "dotenv";
import { createInterface } from "readline/promises";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

const mcc = {
  prim: "pair",
  args: [
    { prim: "nat", annots: ["%token_id"] },
    { prim: "string", annots: ["%source_chain"] },
    { prim: "string", annots: ["%dest_chain"] },
    { prim: "address", annots: ["%dest_address"] },
    {
      prim: "or",
      args: [
        { prim: "address", annots: ["%addr"] },
        { prim: "string", annots: ["%str"] },
      ],
      annots: ["%source_nft_contract_address"],
    },
    { prim: "string", annots: ["%name"] },
    { prim: "string", annots: ["%symbol"] },
    { prim: "nat", annots: ["%royalty"] },
    { prim: "address", annots: ["%royalty_receiver"] },
    { prim: "string", annots: ["%metadata"] },
    { prim: "string", annots: ["%transaction_hash"] },
    { prim: "nat", annots: ["%token_amount"] },
    { prim: "string", annots: ["%nft_type"] },
    { prim: "mutez", annots: ["%fee"] },
  ],
  annots: ["%data"],
} as MichelsonType;

export async function get_token_metadata() {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  const signer = await InMemorySigner.fromSecretKey(SK);
  const schema = new Schema(mcc);
  const data = {
    token_id: 25,
    source_chain: "TEZOS",
    dest_chain: "TEZOS",
    dest_address: "tz1hmsQEAzt7F1y7X6xjv1U4pqk4xeKKKPcR",
    source_nft_contract_address: "tz1NqGupPkw59EHtc6K1YfMqrmgHAjz4qWoB",
    name: "NTEZOS",
    symbol: "NTEZOS",
    royalty: 150,
    royalty_receiver: "tz1NqGupPkw59EHtc6K1YfMqrmgHAjz4qWoB",
    metadata:
      "https://ipfs.io/ipfs/QmToNVaaxKDPZnJmBXXCcFXbTyBmVzmKtoRcQBzsfTNbUT",
    transaction_hash:
      "0x99b137f6b68d76035266ad0c949891ccea82989dbf2bd2340d0a8102a6b36166",
    token_amount: 1,
    nft_type: "singular",
    fee: 10000,
  };
  console.log(
    packDataBytes(
      {
        string: "0xa702b3873c6818de60d1495b792eefd1e4ebb2a2",
      },
      {
        prim: "string",
      }
    ).bytes
  );
  const data2 = {
    token_id: 25,
    source_chain: "BSC",
    dest_chain: "TEZOS",
    dest_address: "tz1hmsQEAzt7F1y7X6xjv1U4pqk4xeKKKPcR",
    source_nft_contract_address: {
      str: "0xa702b3873c6818de60d1495b792eefd1e4ebb2a2",
    },
    name: "DUM",
    symbol: "D",
    royalty: 0,
    royalty_receiver: "tz1NqGupPkw59EHtc6K1YfMqrmgHAjz4qWoB",
    metadata:
      "https://ipfs.io/ipfs/QmToNVaaxKDPZnJmBXXCcFXbTyBmVzmKtoRcQBzsfTNbUT",
    transaction_hash:
      "0xfc20f11690da4c77dbd9c2ab243c3c66d489ffb3f972646cfe88d237167a36fa",
    token_amount: 1,
    nft_type: "singular",
    fee: 10000,
  };
  const encoded = schema.Encode(data2);
  const packedData = packDataBytes(encoded, mcc);
  const packeyBytes = packedData.bytes;
  console.log(`PACK:`, packeyBytes);
  console.log(`MSG:`, keccak256(Buffer.from(packeyBytes, "hex")));
  console.log(`PUBK:`, await signer.publicKey());

  const signature = await signer.sign(
    keccak256(Buffer.from(packeyBytes, "hex"))
  );
  console.log(`SIG:`, signature.sig);
  console.log("ADDR: ", await signer.publicKeyHash());
}

if (require.main === module) {
  (async () => {
    await get_token_metadata();
  })();
}
