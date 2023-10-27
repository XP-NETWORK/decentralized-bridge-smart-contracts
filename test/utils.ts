import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { EventLog, Typed, ethers, keccak256 } from "ethers";
import { ethers as hardhatEthers } from "hardhat";
import {
    TCreateHashReturn,
    TLockedEventData,
    TNFTDetails,
    TNFTType,
    TProcessedLogs,
} from "./types";

const encoder = new ethers.AbiCoder();
export const FEE = ethers.Typed.uint256(5);
export const AMOUNT_TO_LOCK = 1;

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
        fee: FEE.value,
    };

    const nftTransferDetailsValues = Object.values(claimDataArgs);

    const dataHash = keccak256(
        encoder.encode(NftTransferDetailsTypes, nftTransferDetailsValues)
    );

    const hexifiedDataHash = hexStringToByteArray(dataHash);

    return [claimDataArgs, hexifiedDataHash, hash];
};

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
    const byteArray: number[] = [];
    for (let i = 0; i < hexString.length; i += 2) {
        byteArray.push(parseInt(hexString.substr(i, 2), 16));
    }
    return new Uint8Array(byteArray);
};

export async function deploy721Collection(bscUser: HardhatEthersSigner) {
    const name = "MyCollection";
    const symbol = "MC";

    const CollectionDeployer = await hardhatEthers.getContractFactory(
        "NFTCollectionDeployer"
    );

    const collectionInstance = await CollectionDeployer.connect(
        bscUser
    ).deploy();

    await collectionInstance.setOwner(bscUser.address);

    const response = await collectionInstance
        .deployNFT721Collection(name, symbol)
        .then((r) => r.wait());

    const logs = response!.logs[1] as EventLog;
    const newCollectionAddress = logs.args[0];

    const mintedCollectionOnBSC = await hardhatEthers.getContractAt(
        "ERC721Royalty",
        newCollectionAddress
    );

    const mintedCollectionOnBSCAddress =
        await mintedCollectionOnBSC.getAddress();

    const toAddress = bscUser.address;
    const tokenId1 = ethers.Typed.uint256(1);
    const tokenId2 = ethers.Typed.uint256(2);
    const tokenIds: [Typed, Typed] = [tokenId1, tokenId2];
    const royalty = ethers.Typed.uint256(100);
    const royaltyReceiver = bscUser.address;
    const tokenURI = "";

    const nftDetails = {
        toAddress,
        tokenId1,
        tokenId2,
        royalty,
        royaltyReceiver,
        tokenURI,
        collectionAddress: newCollectionAddress,
        name,
        symbol,
    };

    const mintPromises = tokenIds.map((id) => {
        const mintArgs: Parameters<typeof mintedCollectionOnBSC.mint> = [
            toAddress,
            id,
            royalty,
            royaltyReceiver,
            tokenURI,
        ];
        return mintedCollectionOnBSC.connect(bscUser).mint(...mintArgs);
    });

    await Promise.all(mintPromises);

    return {
        mintedCollectionOnBSC,
        mintedCollectionOnBSCAddress,
        nftDetails,
        tokenIds,
    };
}

export async function deploy1155Collection(
    toMint: number,
    bscUser: HardhatEthersSigner
) {
    const name = "MyCollection";
    const symbol = "MC";

    const CollectionDeployer = await hardhatEthers.getContractFactory(
        "NFTCollectionDeployer"
    );

    const collectionInstance = await CollectionDeployer.connect(
        bscUser
    ).deploy();

    await collectionInstance.setOwner(bscUser.address);

    const response = await collectionInstance
        .connect(bscUser)
        .deployNFT1155Collection()
        .then((r) => r.wait());

    const logs = response!.logs[1] as EventLog;
    const newCollectionAddress = logs.args[0];

    const mintedCollectionOnBSC = await hardhatEthers.getContractAt(
        "ERC1155Royalty",
        newCollectionAddress
    );

    const mintedCollectionOnBSCAddress =
        await mintedCollectionOnBSC.getAddress();

    const toAddress = bscUser.address;
    const tokenId1 = ethers.Typed.uint256(1);
    const tokenId2 = ethers.Typed.uint256(2);
    const tokenIds: [Typed, Typed] = [tokenId1, tokenId2];
    const royalty = ethers.Typed.uint256(100);
    const royaltyReceiver = bscUser.address;
    const tokenURI = "";

    const nftDetails = {
        toAddress,
        tokenId1,
        tokenId2,
        royalty,
        royaltyReceiver,
        tokenURI,
        collectionAddress: newCollectionAddress,
        name,
        symbol,
    };

    const mintPromises = tokenIds.map((id) => {
        return mintedCollectionOnBSC
            .connect(bscUser)
            .mint(
                bscUser.address,
                id,
                ethers.Typed.uint256(toMint),
                ethers.Typed.uint256(100),
                bscUser.address,
                ""
            );
    });

    await Promise.all(mintPromises);

    return {
        mintedCollectionOnBSC,
        mintedCollectionOnBSCAddress,
        nftDetails,
        tokenIds,
    };
}

export const getNftType = (nftType: TNFTType) =>
    nftType === 721 ? "singular" : "multiple";
