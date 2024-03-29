import { beforeAll, describe, expect, it } from "@jest/globals";
import { InMemorySigner, generateSecretKey } from "@taquito/signer";
import { packDataBytes, MichelsonType } from "@taquito/michel-codec";
import { Contract, MichelCodecPacker, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { deployBridge } from "../scripts/deploy-bridge-contract";
import { randomBytes } from "crypto";
import { BridgeContractType } from "../types/Bridge.types";
import { address, bytes, key, signature, tas } from "../types/type-aliases";
import { deployNft, mintNft } from "./utils";
import { Schema } from "@taquito/michelson-encoder";

const randomSigner = () =>
  new InMemorySigner(
    generateSecretKey(randomBytes(32), "44'/1729'/0'/0'", "ed25519")
  );

describe("Bridge Contract Tests", () => {
  let RPC: string = process.env.RPC_ENDPOINT!;
  let SIGNER_PK: string = process.env.SK!;
  let Tezos: TezosToolkit;
  let ca: string | undefined;
  let val1: InMemorySigner;
  let val2: InMemorySigner;
  let val3: InMemorySigner;
  let val4: InMemorySigner;
  let val5: InMemorySigner;
  beforeAll(async () => {
    config();
    RPC = process.env.RPC_ENDPOINT!;
    SIGNER_PK = process.env.SK!;
    Tezos = new TezosToolkit(RPC);
    Tezos.setSignerProvider(new InMemorySigner(SIGNER_PK));
    val1 = randomSigner();
    val2 = randomSigner();
    val3 = randomSigner();
    val4 = randomSigner();
    val5 = randomSigner();
  });

  it("Should successfully deploy the bridge contract", async () => {
    ca = await deployBridge([
      await val1.publicKeyHash(),
      await val2.publicKeyHash(),
      await val3.publicKeyHash(),
      await val4.publicKeyHash(),
      await val5.publicKeyHash(),
    ]);
    expect(ca).toBeDefined();
    const ctr = await Tezos.contract.at<BridgeContractType>(ca!);
    const validators = await ctr.storage();
    expect(validators.validators_count).toEqual(5);
  });

  it("Add Validators with atleast 4 signers", async () => {
    expect(ca).toBeDefined();
    const ctr = await Tezos.contract.at<BridgeContractType>(ca!);
    const new_val = randomSigner();
    const packer = new MichelCodecPacker();
    const packed = await packer.packData({
      data: {
        string: await Tezos.signer.publicKeyHash(),
      },
      type: {
        prim: "address",
      },
    });

    const added = ctr.methods.add_validator(
      tas.address(await new_val.publicKeyHash()),
      await Promise.all(
        [val1, val2, val3, val4, val5].map(async (s) => {
          return {
            addr: tas.address(await s.publicKeyHash()),
            signer: (await s.publicKey()) as key,
            sig: (await s.sign(packed.packed)).sig as signature,
          };
        })
      )
    );
    expect(added).toBeDefined();
  });

  it("Lock NFT", async () => {
    expect(ca).toBeDefined();
    const [nftA, nftC] = await deployNft(Tezos);
    const minted = await mintNft(nftC, 0, await Tezos.signer.publicKeyHash());
    expect(minted).toBeDefined();
    // Approve NFT
    const approval = await (nftC as unknown as Contract).methodsObject
      .update_operators([
        {
          add_operator: {
            owner: tas.address(await Tezos.signer.publicKeyHash()),
            operator: tas.address(ca!),
            token_id: tas.nat(0),
          },
        },
      ])
      .send();
    await approval.confirmation();

    const ctr = await Tezos.contract.at<BridgeContractType>(ca!);

    const added = await ctr.methods
      .lock_nft(tas.nat(0), "NSC", "BSCADDRESS", nftA as address)
      .send();
    await added.confirmation();
    expect(added).toBeDefined();
    const storage = await ctr.storage();
    const address = await storage.original_storage_mapping_nft.get({
      "0": tas.address(nftA),
      "1": "TEZOS",
    });
    expect(address).toBeDefined();
  });

  it("Claim NFT", async () => {
    expect(ca).toBeDefined();
    const data = {
      token_id: tas.nat(1),
      dest_address: tas.address(await Tezos.signer.publicKeyHash()),
      dest_chain: "TEZOS",
      fee: tas.mutez(0),
      metadata: "METADATA",
      name: "NAME",
      nft_type: "NFT",
      royalty: tas.nat(0),
      royalty_receiver: tas.address(await Tezos.signer.publicKeyHash()),
      sigs: [],
      source_chain: "BSC",
      source_nft_contract_address: "0x" as bytes,
      symbol: "SYMBOL",
      token_amount: tas.nat(0),
      transaction_hash: "HASH",
    };
    const cda = await generate_claim_data_bytes(data);
    const sigs: {
      signer: key;
      sig: signature;
      addr: address;
    }[] = await Promise.all(
      [val1, val2, val3, val4, val5].map(async (e) => {
        return {
          addr: tas.address(await e.publicKeyHash()),
          signer: (await e.publicKey()) as key,
          sig: (await e.sign(cda)).sig as signature,
        };
      })
    );
    const ctr = await Tezos.contract.at<BridgeContractType>(ca!);
    const claim = await ctr.methods
      .claim_nft(
        data.token_id,
        data.source_chain,
        data.dest_chain,
        data.dest_address,
        data.source_nft_contract_address,
        data.name,
        data.symbol,
        data.royalty,
        data.royalty_receiver,
        data.metadata,
        data.transaction_hash,
        data.token_amount,
        data.nft_type,
        data.fee,
        sigs
      )
      .send();
    expect(claim).toBeDefined();
  });
});
type cda = Parameters<BridgeContractType["methodsObject"]["claim_nft"]>[0];
const generate_claim_data_bytes = async ({
  token_id,
  dest_address,
  dest_chain,
  fee,
  metadata,
  name,
  nft_type,
  royalty,
  royalty_receiver,
  source_chain,
  source_nft_contract_address,
  symbol,
  token_amount,
  transaction_hash,
}: cda) => {
  const mcc = {
    prim: "pair",
    args: [
      { prim: "nat", annots: ["%token_id"] },
      { prim: "string", annots: ["%source_chain"] },
      { prim: "string", annots: ["%dest_chain"] },
      { prim: "address", annots: ["%dest_address"] },
      {
        prim: "bytes",
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
  const schema = new Schema(mcc);
  const encoded = schema.Encode({
    token_id,
    source_chain,
    dest_chain,
    dest_address,
    source_nft_contract_address,
    name,
    symbol,
    royalty,
    royalty_receiver,
    metadata,
    transaction_hash,
    token_amount,
    nft_type,
    fee,
  });
  const packedData = packDataBytes(encoded, mcc);
  return packedData.bytes;
};
