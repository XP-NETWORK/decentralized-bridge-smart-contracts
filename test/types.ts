import { Contract, ContractTransactionReceipt, Typed } from "ethers";
import { Bridge, ERC721Royalty } from "../contractsTypes";

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

export type TNFTType = 721 | 1155;
export type TContractInstance = Bridge;

export type TBridge = {
    bridge: TContractInstance;
    address: string;
    chainSymbol: string;
    collectionDeployer: string;
    storageDeployer: string;
};

export type TGetValidatorSignatures = (
    hash: Uint8Array,
    type: "eth" | "bsc"
) => Promise<[string, string]>;
