import { describe, it, expect, beforeAll } from "@jest/globals";
import { InMemorySigner } from "@taquito/signer";
import { Contract, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";

import { tas } from "../types/type-aliases";
import { deployNft, mintNft } from "./utils";
import { NFTContractType, Storage } from "../types/NFT.types";
import { NFTStorageContractType } from "../types/NFTStorage.types";
import { deployNftStorage } from "../scripts/deploy-nft-storage";
import { deployCollectionFactory } from "../scripts/deploy-collection-factory";
import { CollectionFactoryContractType } from "../types/CollectionFactory.types";

describe("Collection Deployer Contract", () => {
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
    const sf = await deployCollectionFactory(
      await Tezos.signer.publicKeyHash()
    );
    expect(sf).toBeDefined();
    ca = sf!;
  });

  it("Creating a NFT Collection", async () => {
    let ctr = await Tezos.contract.at<CollectionFactoryContractType>(ca);
    const deploy_tx = await ctr.methods.deploy_nft(tas.address(ca)).send();
    await deploy_tx.confirmation();
    const storage = await ctr.storage();
    const deployed = storage.collection_to_store.get(tas.address(ca));
    expect(deployed).toBeDefined();
  });

  it("Creating a SFT Collection", async () => {
    let ctr = await Tezos.contract.at<CollectionFactoryContractType>(ca);
    const deploy_tx = await ctr.methods.deploy_sft(tas.address(ca)).send();
    await deploy_tx.confirmation();
    const storage = await ctr.storage();
    const deployed = storage.collection_to_store.get(tas.address(ca));
    expect(deployed).toBeDefined();
  });
});
