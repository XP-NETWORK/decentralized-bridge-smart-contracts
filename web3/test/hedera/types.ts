import { Contract, ContractTransactionReceipt, Typed, Wallet } from "ethers";
import {
  Bridge,
  ERC1155Royalty,
  ERC721Royalty,
  HederaBridge,
} from "../../contractsTypes";

export type TLockedEventData = {
  tokenId: number;
  destinationChain: any;
  destinationUserAddress: any;
  sourceNftContractAddress: any;
  tokenAmount: number;
  nftType: any;
  sourceChain: any;
};

export type TNFTDetails = {
  toAddress: string;
  tokenId1: Typed;
  tokenId2: Typed;
  royalty: Typed;
  royaltyReceiver: string;
  tokenURI: string;
  collectionAddress: string;
  name: string;
  symbol: string;
  chainSymbolOfNFT: string;
};

export type TProcessedLogs = {
  tokenId: number;
  destinationChain: string;
  destinationUserAddress: string;
  sourceNftContractAddress: string;
  tokenAmount: number;
  nftType: string;
  sourceChain: string;
};

export type TLockOnBSCAndClaimOnEthReturn = Promise<
  [
    logs: TProcessedLogs[],
    storageAddresses: string[],
    storageContracts: Contract[]
  ]
>;

export type TCreateHashReturn = [
  Parameters<Bridge["claimNFT721"]>[0],
  Uint8Array,
  string
];

export type TLockReturn = [
  TProcessedLogs,
  TProcessedLogs,
  ContractTransactionReceipt | null,
  ContractTransactionReceipt | null
];

export type TLockReturn2 = [
  [TProcessedLogs, TProcessedLogs],
  ContractTransactionReceipt | null,
  ContractTransactionReceipt | null
];

export type TNFTType = 721 | 1155;
export type TContractInstance = HederaBridge;

export type THederaBridge = {
  bridge: TContractInstance;
  address: string;
  chainSymbol: string;
  storageDeployer: string;
};

export type TGetValidatorSignatures = (
  hash: Uint8Array,
  type: "eth" | "bsc"
) => Promise<[string, string]>;

export type TLockOnBSCAndClaimOnEthArgs = {
  mintedCollectionOnBSC: ERC1155Royalty | ERC721Royalty;
  tokenIds: [Typed, Typed];
  mintedCollectionOnBSCAddress: string;
  nftDetails: TNFTDetails;
  bscUser: Wallet;
  ethUser: Wallet;
  bscBridge: THederaBridge;
  ethBridge: THederaBridge;
  nftType: TNFTType;
  getValidatorSignatures: TGetValidatorSignatures;
  amountToLock?: number;
};

export type TLockOnEthAndClaimOnBSCArgs = {
  lockedEventDatas: TProcessedLogs[];
  duplicateCollectionContracts: Contract[];
  duplicateCollectionAddresses: string[];
  mintedCollectionOnBSC: ERC721Royalty | ERC1155Royalty;
  mintedCollectionOnBSCAddress: string;
  nftDetails: TNFTDetails;
  bscUser: Wallet;
  ethUser: Wallet;
  bscBridge: THederaBridge;
  ethBridge: THederaBridge;
  getValidatorSignatures: TGetValidatorSignatures;
  nftType: TNFTType;
  amountToLock?: number;
};

export type THederaChainArr = {
  chainId: string;
  bridge: THederaBridge | null;
  validatorSet: [Wallet, Wallet];
  deployer: Wallet;
  user: Wallet;
};

export type TChainArrWithBridge = {
  chainId: string;
  bridge: THederaBridge;
  validatorSet: [Wallet, Wallet];
  deployer: Wallet;
  user: Wallet;
};
