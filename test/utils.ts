import { EventLog, ethers, keccak256 } from "ethers";
import {
    TCreateHashReturn,
    TLockedEventData,
    TNFTDetails,
    TProcessedLogs,
} from "./types";

const encoder = new ethers.AbiCoder();
const FEE = 5;
const NftTransferDetailsTypes = [
    "uint256", // 0 - tokenId
    "string", // 1 - sourceChain
    "string", // 2 - destinationChain
    "address", // 3 - destinationUserAddress
    "address", // 4 - sourceNftContractAddress
    "string", // 5 - name
    "string", // 6 - symbol
    "uint256", // 7 - royalty
    "address", // 8 - royaltyReceiver
    "string", // 9 - metadata
    "string", // 10 - transactionHash
    "uint256", // 11 - tokenAmount
    "string", // 12 - nftType
    "uint256", // 13 - fee
];

export const createHash = function (
    lockedEventData: TLockedEventData,
    hash: string = "",
    nftDetails: TNFTDetails,
    royaltyReceiver: string
): TCreateHashReturn {
    const claimDataArgs = {
        tokenId: lockedEventData.tokenId,
        sourceChain: lockedEventData.sourceChain,
        destinationChain: lockedEventData.destinationChain,
        destinationUserAddress: lockedEventData.destinationUserAddress,
        sourceNftContractAddress: lockedEventData.sourceNftContractAddress,
        name: nftDetails.name,
        symbol: nftDetails.symbol,
        royalty: nftDetails.royalty.value,
        royaltyReceiver,
        metadata: nftDetails.tokenURI,
        transactionHash: hash ?? "",
        tokenAmount: lockedEventData.tokenAmount,
        nftType: lockedEventData.nftType,
        fee: ethers.Typed.uint256(FEE).value,
    };

    const nftTransferDetailsValues = Object.values(claimDataArgs);

    const dataHash = keccak256(
        encoder.encode(NftTransferDetailsTypes, nftTransferDetailsValues)
    );

    const hexifiedDataHash = hexStringToByteArray(dataHash);

    return [claimDataArgs, hexifiedDataHash, hash];
};

export const Fee = ethers.Typed.uint256(FEE);

export const parseLogs = (logs: EventLog): TProcessedLogs => {
    const lockedOnEthLogsArgs = logs.args;
    return {
        tokenId: Number(lockedOnEthLogsArgs[0]),
        destinationChain: lockedOnEthLogsArgs[1],
        destinationUserAddress: lockedOnEthLogsArgs[2],
        sourceNftContractAddress: lockedOnEthLogsArgs[3],
        tokenAmount: Number(lockedOnEthLogsArgs[4]),
        nftType: lockedOnEthLogsArgs[5],
        sourceChain: lockedOnEthLogsArgs[6],
    };
};

export const hexStringToByteArray = (hexString: string) => {
    if (hexString.startsWith("0x")) {
        hexString = hexString.slice(2);
    }
    const byteArray = [];
    for (let i = 0; i < hexString.length; i += 2) {
        byteArray.push(parseInt(hexString.substr(i, 2), 16));
    }
    return new Uint8Array(byteArray);
};
