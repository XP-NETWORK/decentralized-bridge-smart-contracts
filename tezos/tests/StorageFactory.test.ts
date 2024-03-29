import { describe, it, expect, beforeAll } from "@jest/globals";
import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { deployStorageFactory } from "../scripts/deploy-storage-factory";

import { StorageFactoryContractType } from "../types/StorageFactory.types";
import { address } from "../types/type-aliases";

describe("Collection Deployer", () => {
  const CC_NFT = "KT1TUQA3eQtXa7Pc9UhK81ZBQnsRZhyYy8J8";
  const CC_SFT = "KT1B979ku773RHqmjR87m9oJxuVyKuL3H1oD";
  let RPC: string = process.env.RPC_ENDPOINT!;
  let SIGNER_PK: string = process.env.SK!;
  let Tezos: TezosToolkit;
  let ca: string;
  beforeAll(() => {
    config();
    RPC = process.env.RPC_ENDPOINT!;
    SIGNER_PK = process.env.SK!;
    Tezos = new TezosToolkit(RPC);
    Tezos.setSignerProvider(new InMemorySigner(SIGNER_PK));
  });

  it("Should deploy a collection", async () => {
    const sf = await deployStorageFactory(await Tezos.signer.publicKeyHash());
    expect(sf).toBeDefined();
    ca = sf!;
  });

  it("Deploy NFT Storage", async () => {
    const contract = await Tezos.contract.at<StorageFactoryContractType>(ca);

    const response = await contract.methods
      .deploy_nft_storage(CC_NFT as address)
      .send();
    await response.confirmation();
    const storage = (await contract.storage()).collection_to_store.get(
      CC_NFT as address
    );
    expect(storage).toBeDefined();
  });
  it("Deploy SFT Storage", async () => {
    const contract = await Tezos.contract.at<StorageFactoryContractType>(ca);

    const response = await contract.methods
      .deploy_sft_storage(CC_SFT as address)
      .send();
    await response.confirmation();
    const storage = (await contract.storage()).collection_to_store.get(
      CC_SFT as address
    );
    expect(storage).toBeDefined();
  });
});
