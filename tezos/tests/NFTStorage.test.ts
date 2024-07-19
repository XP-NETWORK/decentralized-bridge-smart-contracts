import { describe, it, expect, beforeAll } from "@jest/globals";
import { InMemorySigner } from "@taquito/signer";
import { Contract, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";

import { tas } from "../types/type-aliases";
import { deployNft, mintNft } from "./utils";
import { NFTContractType, Storage } from "../types/NFT.types";
import { NFTStorageContractType } from "../types/NFTStorage.types";
import { deployNftStorage } from "../scripts/deploy-nft-storage";

describe("NFT Storage Contract", () => {
  let RPC: string = process.env.RPC_ENDPOINT!;
  let SIGNER_PK: string = process.env.SK!;
  let Tezos: TezosToolkit;
  let ca: string;
  let nft: string;
  let nftContract: NFTContractType;
  beforeAll(() => {
    config();
    RPC = process.env.RPC_ENDPOINT!;
    SIGNER_PK = process.env.SK!;
    Tezos = new TezosToolkit(RPC);
    Tezos.setSignerProvider(new InMemorySigner(SIGNER_PK));
  });

  it("Should deploy a collection", async () => {
    [nft, nftContract] = await deployNft(Tezos);

    const sf = await deployNftStorage({
      collection: nft,
      owner: await Tezos.signer.publicKeyHash(),
    });
    expect(sf).toBeDefined();
    ca = sf!;
  });

  it("Try Locking an NFT", async () => {
    const tokenId = 0;
    // Mint NFT
    await mintNft(
      nftContract as NFTContractType,
      tokenId,
      await Tezos.signer.publicKeyHash()
    );
    // Approve NFT
    const approval = await (nftContract as unknown as Contract).methodsObject
      .update_operators([
        {
          add_operator: {
            owner: tas.address(await Tezos.signer.publicKeyHash()),
            operator: tas.address(ca),
            token_id: tas.nat(tokenId),
          },
        },
      ])
      .send();
    await approval.confirmation();

    const contract = await Tezos.contract.at<NFTStorageContractType>(ca);
    // Lock NFT
    const response = await contract.methods
      .deposit_token(tas.nat(tokenId))
      .send();
    await response.confirmation();
    const owner = await (
      await nftContract.storage<Storage>()
    ).ledger.get(tas.nat(tokenId));
    expect(owner).toEqual(ca);
  });
  it("Try Unlocking an NFT", async () => {
    const tokenId = 0;
    const contract = await Tezos.contract.at<NFTStorageContractType>(ca);

    const response = await contract.methods
      .unlock_token(
        tas.nat(tokenId),
        tas.address(await Tezos.signer.publicKeyHash())
      )
      .send();
    await response.confirmation();
    const owner = await (
      await nftContract.storage<Storage>()
    ).ledger.get(tas.nat(tokenId));
    expect(owner).toEqual(await Tezos.signer.publicKeyHash());
  });
});
