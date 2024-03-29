import {
  get_contract_metadata,
  get_token_metadata,
} from "./get-token-metadata";

export interface INftContract {
  name: (tokenId?: bigint) => Promise<string>;
  symbol: (tokenId?: bigint) => Promise<string>;
  royaltyInfo: (tokenId?: bigint) => Promise<string>;
  tokenURI: (tokenId: bigint) => Promise<string>;
}

export async function constructNftContractInterface({
  contractAddress,
}: {
  contractAddress: string;
}): Promise<INftContract> {
  return {
    name: async (tokenId?: bigint) => {
      const md = await get_contract_metadata(contractAddress);
      return (await md.metadataName()) ?? "NFT NAME NOT FOUND";
    },
    symbol: async (tokenId?: bigint) => {
      const mdOrUrl = await get_token_metadata(tokenId!, contractAddress);
      const isUrl = URL.canParse(mdOrUrl);
      if (isUrl) {
        const md: any = await fetch(mdOrUrl).then((res) => res.json());
        return md.symbol;
      }
      return JSON.parse(mdOrUrl)["symbol"];
    },
    royaltyInfo: async (tokenId?: bigint) => {
      const mdOrUrl = await get_token_metadata(tokenId!, contractAddress);
      const isUrl = URL.canParse(mdOrUrl);
      if (isUrl) {
        const md: any = await fetch(mdOrUrl).then((res) => res.json());
        const royalties = md.royalties;
        return Object.keys(royalties.shares)[0];
      }
      const royalties = JSON.parse(mdOrUrl)["royalties"];
      return Object.keys(royalties.shares)[0];
    },
    tokenURI: async (tokenId: bigint) => {
      const urlOrMd = await get_token_metadata(tokenId!, contractAddress);
      return URL.canParse(urlOrMd) ? urlOrMd : JSON.parse(urlOrMd)["image"];
    },
  };
}
