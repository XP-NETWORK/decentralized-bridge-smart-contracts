import { MichelsonMap, TezosToolkit } from "@taquito/taquito";
import NFT from "../build/NFT.json";
import { NFTContractType } from "../types/NFT.types";
import { tas } from "../types/type-aliases";
import { MultiAssetCode } from "../types/MultiAsset.code";
import { MultiAssetContractType } from "../types/MultiAsset.types";

export async function deployNft(Tezos: TezosToolkit) {
  const address = await Tezos.contract.originate({
    code: NFT,
    storage: {
      ledger: new MichelsonMap(),
      operators: new MichelsonMap(),
      token_metadata: new MichelsonMap(),
      metadata: new MichelsonMap(),
      admin: await Tezos.signer.publicKeyHash(),
    },
  });
  await address.confirmation();
  return [
    address.contractAddress!,
    await Tezos.contract.at<NFTContractType>(address.contractAddress!),
  ] as const;
}

export async function mintNft(
  contract: NFTContractType,
  tokenId: number,
  owner: string
) {
  const response = await contract.methods
    .mint([
      {
        amt: tas.nat(1),
        to: tas.address(owner),
        token_id: tas.nat(tokenId),
        token_uri: "https://meta.polkamon.com/meta?id=10001852306",
      },
    ])
    .send();
  await response.confirmation();
  return tokenId;
}

export async function deploySft(Tezos: TezosToolkit) {
  const address = await Tezos.contract.originate({
    code: MultiAssetCode.code,
    storage: {
      ledger: new MichelsonMap(),
      operators: new MichelsonMap(),
      token_metadata: new MichelsonMap(),
      metadata: new MichelsonMap(),
      admin: await Tezos.signer.publicKeyHash(),
    },
  });
  await address.confirmation();
  return [
    address.contractAddress!,
    await Tezos.contract.at<NFTContractType>(address.contractAddress!),
  ] as const;
}

export async function mintSft(
  contract: MultiAssetContractType,
  tokenId: number,
  owner: string,
  amt: number | string
) {
  const response = await contract.methods
    .mint(
      tas.nat(tokenId),
      "https://meta.polkamon.com/meta?id=10001852306",
      tas.address(owner),
      tas.nat(amt)
    )
    .send();
  await response.confirmation();
  return tokenId;
}
