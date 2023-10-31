import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import {
    Contract,
    ContractTransactionReceipt,
    EventLog,
    Typed,
    ZeroAddress,
    ethers,
    keccak256,
} from "ethers";
import { ethers as hardhatEthers } from "hardhat";
import { ERC1155Royalty, ERC721Royalty } from "../contractsTypes";
import {
    TBridge,
    TCreateHashReturn,
    TGetValidatorSignatures,
    TLockOnBSCAndClaimOnEthArgs,
    TLockOnBSCAndClaimOnEthReturn,
    TLockOnEthAndClaimOnBSCArgs,
    TLockReturn,
    TLockReturn_NEW,
    TLockedEventData,
    TNFTDetails,
    TNFTType,
    TProcessedLogs,
} from "./types";

export const encoder = new ethers.AbiCoder();
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

export async function lockOnBSC(
    mintedCollectionOnBSC: ERC1155Royalty | ERC721Royalty,
    tokenIds: [Typed, Typed],
    mintedCollectionOnBSCAddress: string,
    nftDetails: TNFTDetails,
    bscUser: HardhatEthersSigner,
    ethUser: HardhatEthersSigner,
    bscBridge: TBridge,
    ethBridge: TBridge,
    nftType: TNFTType
): Promise<TLockReturn> {
    await Promise.all(
        tokenIds.map(async (id) => {
            if (nftType === 721) {
                return (mintedCollectionOnBSC as ERC721Royalty)
                    .connect(bscUser)
                    .approve(bscBridge.address, id)
                    .then((r) => r.wait());
            } else {
                return (mintedCollectionOnBSC as ERC1155Royalty)
                    .connect(bscUser)
                    .setApprovalForAll(bscBridge.address, true)
                    .then((r) => r.wait());
            }
        })
    );
    /// lock the nft by creating a new storage
    const [lockedReceipt1, lockedReceipt2] = await Promise.all(
        tokenIds.map(async (id) => {
            if (nftType === 721) {
                return bscBridge.bridge
                    .connect(bscUser)
                    .lock721(
                        id,
                        ethBridge.chainSymbol,
                        ethUser.address,
                        nftDetails.collectionAddress
                    )
                    .then((r) => r.wait());
            } else {
                return bscBridge.bridge
                    .connect(bscUser)
                    .lock1155(
                        id,
                        ethBridge.chainSymbol,
                        ethUser.address,
                        nftDetails.collectionAddress,
                        ethers.Typed.uint256(AMOUNT_TO_LOCK)
                    )
                    .then((r) => r.wait());
            }
        })
    );

    // get the new storage contract address for the original nft
    const storageAddressForCollection = await bscBridge.bridge[
        nftType === 721
            ? "originalStorageMapping721"
            : "originalStorageMapping1155"
    ](nftDetails.collectionAddress, "BSC");
    expect(storageAddressForCollection).to.not.be.equal(ethers.ZeroAddress);

    if (nftType === 721) {
        const [owner1, owner2] = await Promise.all([
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                nftDetails.tokenId1
            ),
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                nftDetails.tokenId2
            ),
        ]);
        expect(storageAddressForCollection).to.be.equal(owner1);
        expect(storageAddressForCollection).to.be.equal(owner2);
    } else {
        const [balance1, balance2] = await Promise.all([
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                storageAddressForCollection,
                nftDetails.tokenId1
            ),
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                storageAddressForCollection,
                nftDetails.tokenId2
            ),
        ]);
        // Check if storageAddressForCollection has at least one of each token
        expect(balance1).to.be.gt(0);
        expect(balance2).to.be.gt(0);
    }

    // =================================================================
    const parsedLogs1 = parseLogs(lockedReceipt1?.logs[1] as EventLog);
    const parsedLogs2 = parseLogs(lockedReceipt2?.logs[1] as EventLog);

    expect(parsedLogs1.tokenId).to.be.equal(nftDetails.tokenId1.value);
    expect(parsedLogs1.destinationChain).to.be.equal(ethBridge.chainSymbol);
    expect(parsedLogs1.destinationUserAddress).to.be.equal(ethUser.address);
    expect(parsedLogs1.sourceNftContractAddress).to.be.equal(
        mintedCollectionOnBSCAddress
    );
    expect(parsedLogs1.tokenAmount).to.be.equal(1);
    expect(parsedLogs1.nftType).to.be.equal(getNftType(nftType));
    expect(parsedLogs1.sourceChain).to.be.equal(bscBridge.chainSymbol);

    expect(parsedLogs2.tokenId).to.be.equal(nftDetails.tokenId2.value);
    expect(parsedLogs2.destinationChain).to.be.equal(ethBridge.chainSymbol);
    expect(parsedLogs2.destinationUserAddress).to.be.equal(ethUser.address);
    expect(parsedLogs2.sourceNftContractAddress).to.be.equal(
        mintedCollectionOnBSCAddress
    );
    expect(parsedLogs2.tokenAmount).to.be.equal(1);
    expect(parsedLogs2.nftType).to.be.equal(getNftType(nftType));
    expect(parsedLogs2.sourceChain).to.be.equal(bscBridge.chainSymbol);

    // @ts-ignore
    return [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2];
}

export async function claimOnEth(
    parsedLogs1: TProcessedLogs,
    parsedLogs2: TProcessedLogs,
    lockedReceipt1: ContractTransactionReceipt | null,
    lockedReceipt2: ContractTransactionReceipt | null,
    nftDetails: TNFTDetails,
    ethUser: HardhatEthersSigner,
    ethBridge: TBridge,
    nftType: TNFTType,
    getValidatorSignatures: TGetValidatorSignatures
): TLockOnBSCAndClaimOnEthReturn {
    const lockedEventDatas = [parsedLogs1, parsedLogs2];

    const lockHashBSC1 = lockedReceipt1?.hash;
    const lockHashBSC2 = lockedReceipt2?.hash;
    const txHashes = [lockHashBSC1, lockHashBSC2];

    await Promise.all(
        lockedEventDatas.map(async (d, i) => {
            const [data, hash] = createHash(
                d,
                txHashes[i],
                nftDetails,
                ethUser.address
            );

            const signatures = await getValidatorSignatures(hash, "eth");

            return ethBridge.bridge
                .connect(ethUser)
                [nftType === 721 ? "claimNFT721" : "claimNFT1155"](
                    data,
                    signatures,
                    {
                        value: FEE.value,
                    }
                )
                .then((r) => r.wait());
        })
    );

    const [
        [destinationChainId1, duplicateCollectionAddress1],
        [destinationChainId2, duplicateCollectionAddress2],
    ] = await Promise.all(
        lockedEventDatas.map((d) =>
            ethBridge.bridge.originalToDuplicateMapping(
                d.sourceNftContractAddress,
                d.sourceChain
            )
        )
    );

    expect(duplicateCollectionAddress1).to.not.be.equal(ZeroAddress);
    expect(duplicateCollectionAddress2).to.not.be.equal(ZeroAddress);
    expect(destinationChainId1).to.be.equal(ethBridge.chainSymbol);
    expect(destinationChainId2).to.be.equal(ethBridge.chainSymbol);

    const duplicateCollectionAddresses = [
        duplicateCollectionAddress1,
        duplicateCollectionAddress2,
    ];

    const duplicateCollectionContracts = await Promise.all(
        duplicateCollectionAddresses.map((contractAddress) =>
            hardhatEthers.getContractAt(
                nftType === 721 ? "ERC721Royalty" : "ERC1155Royalty",
                contractAddress
            )
        )
    );

    const [duplicateCollectionContract1, duplicateCollectionContract2] =
        duplicateCollectionContracts as unknown as
            | ERC721Royalty[]
            | ERC1155Royalty[];

    if (nftType === 721) {
        const duplicateNFTOwnerProm1 = (
            duplicateCollectionContract1 as ERC721Royalty
        ).ownerOf(ethers.Typed.uint256(parsedLogs1.tokenId));

        const duplicateNFTOwnerProm2 = (
            duplicateCollectionContract2 as ERC721Royalty
        ).ownerOf(ethers.Typed.uint256(parsedLogs2.tokenId));

        const royaltyInfoProm1 = duplicateCollectionContract1.royaltyInfo(
            ethers.Typed.uint256(parsedLogs1.tokenId),
            ethers.Typed.uint256(1)
        );

        const royaltyInfoProm2 = duplicateCollectionContract2.royaltyInfo(
            ethers.Typed.uint256(parsedLogs2.tokenId),
            ethers.Typed.uint256(1)
        );

        const [
            duplicateNFTOwner1,
            royaltyInfo1,
            duplicateNFTOwner2,
            royaltyInfo2,
        ] = await Promise.all([
            duplicateNFTOwnerProm1,
            royaltyInfoProm1,
            duplicateNFTOwnerProm2,
            royaltyInfoProm2,
        ]);

        expect(duplicateNFTOwner1).to.be.equal(ethUser.address);
        expect(royaltyInfo1[0]).to.be.equal(ethUser.address); // receiver
        expect(royaltyInfo1[1]).to.be.equal(nftDetails.royalty.value); // value
        expect(duplicateNFTOwner2).to.be.equal(ethUser.address);
        expect(royaltyInfo2[0]).to.be.equal(ethUser.address); // receiver
        expect(royaltyInfo2[1]).to.be.equal(nftDetails.royalty.value); // value
    } else {
        const duplicateNFTOwnerProm1 = (
            duplicateCollectionContract1 as ERC1155Royalty
        ).balanceOf(ethUser.address, ethers.Typed.uint256(parsedLogs1.tokenId));

        const duplicateNFTOwnerProm2 = (
            duplicateCollectionContract2 as ERC1155Royalty
        ).balanceOf(ethUser.address, ethers.Typed.uint256(parsedLogs2.tokenId));

        const royaltyInfoProm1 = duplicateCollectionContract1.royaltyInfo(
            ethers.Typed.uint256(parsedLogs1.tokenId),
            ethers.Typed.uint256(1)
        );

        const royaltyInfoProm2 = duplicateCollectionContract2.royaltyInfo(
            ethers.Typed.uint256(parsedLogs2.tokenId),
            ethers.Typed.uint256(1)
        );

        const [
            duplicateNFTOwner1,
            royaltyInfo1,
            duplicateNFTOwner2,
            royaltyInfo2,
        ] = await Promise.all([
            duplicateNFTOwnerProm1,
            royaltyInfoProm1,
            duplicateNFTOwnerProm2,
            royaltyInfoProm2,
        ]);

        expect(duplicateNFTOwner1).to.be.gt(0);
        expect(royaltyInfo1[0]).to.be.equal(ethUser.address); // receiver
        expect(royaltyInfo1[1]).to.be.equal(nftDetails.royalty.value); // value
        expect(duplicateNFTOwner2).to.be.gt(0);
        expect(royaltyInfo2[0]).to.be.equal(ethUser.address); // receiver
        expect(royaltyInfo2[1]).to.be.equal(nftDetails.royalty.value); // value
    }

    return [
        lockedEventDatas,
        duplicateCollectionAddresses,
        duplicateCollectionContracts,
    ];
}

export async function lockOnEth(
    lockedEventDatas: TProcessedLogs[],
    duplicateCollectionContracts: ERC1155Royalty[] | ERC721Royalty[],
    duplicateCollectionAddresses: string[],
    nftDetails: TNFTDetails,
    bscUser: HardhatEthersSigner,
    ethUser: HardhatEthersSigner,
    bscBridge: TBridge,
    ethBridge: TBridge,
    nftType: TNFTType
): Promise<TLockReturn> {
    const [duplicateCollectionAddress1, duplicateCollectionAddress2] =
        duplicateCollectionAddresses;

    await Promise.all(
        lockedEventDatas.map(async (data, i) => {
            if (nftType === 721) {
                return (duplicateCollectionContracts[i] as ERC721Royalty)
                    .connect(ethUser)
                    .approve(ethBridge.address, data.tokenId)
                    .then((r) => r.wait());
            } else {
                return (duplicateCollectionContracts[i] as ERC1155Royalty)
                    .connect(ethUser)
                    .setApprovalForAll(ethBridge.address, true)
                    .then((r) => r.wait());
            }
        })
    );

    const [lockOnEthReceipt1, lockOnEthReceipt2] = await Promise.all(
        lockedEventDatas.map(async (data, i) => {
            if (nftType === 721) {
                return ethBridge.bridge
                    .connect(ethUser)
                    .lock721(
                        data.tokenId,
                        bscBridge.chainSymbol,
                        bscUser.address,
                        duplicateCollectionContracts[i]
                    )
                    .then((r) => r.wait());
            } else {
                return ethBridge.bridge
                    .connect(ethUser)
                    .lock1155(
                        data.tokenId,
                        bscBridge.chainSymbol,
                        bscUser.address,
                        duplicateCollectionContracts[i],
                        AMOUNT_TO_LOCK
                    )
                    .then((r) => r.wait());
            }
        })
    );

    const originalStorageAddressForDuplicateCollectionProm1 = ethBridge.bridge[
        nftType === 721
            ? "originalStorageMapping721"
            : "originalStorageMapping1155"
    ](duplicateCollectionAddress1, bscBridge.chainSymbol);

    const duplicateStorageAddressForDuplicateCollectionProm1 = ethBridge.bridge[
        nftType === 721
            ? "duplicateStorageMapping721"
            : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress1, ethBridge.chainSymbol);

    const originalStorageAddressForDuplicateCollectionProm2 = ethBridge.bridge[
        nftType === 721
            ? "originalStorageMapping721"
            : "originalStorageMapping1155"
    ](duplicateCollectionAddress2, bscBridge.chainSymbol);

    const duplicateStorageAddressForDuplicateCollectionProm2 = ethBridge.bridge[
        nftType === 721
            ? "duplicateStorageMapping721"
            : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress2, ethBridge.chainSymbol);

    const [
        originalStorageAddressForDuplicateCollection1,
        duplicateStorageAddressForDuplicateCollection1,
        originalStorageAddressForDuplicateCollection2,
        duplicateStorageAddressForDuplicateCollection2,
    ] = await Promise.all([
        originalStorageAddressForDuplicateCollectionProm1,
        duplicateStorageAddressForDuplicateCollectionProm1,
        originalStorageAddressForDuplicateCollectionProm2,
        duplicateStorageAddressForDuplicateCollectionProm2,
    ]);

    // ======================= LOCK ON ETH - VERIFY ===================

    expect(originalStorageAddressForDuplicateCollection1).to.be.equal(
        ZeroAddress
    );
    expect(duplicateStorageAddressForDuplicateCollection1).to.not.be.equal(
        ZeroAddress
    );

    expect(originalStorageAddressForDuplicateCollection2).to.be.equal(
        ZeroAddress
    );
    expect(duplicateStorageAddressForDuplicateCollection2).to.not.be.equal(
        ZeroAddress
    );

    /**
     *  emit Locked(
            tokenId,
            destinationChain,
            destinationUserAddress,
            address(originalCollectionAddress.contractAddress),
            1,
            TYPEERC721,
            originalCollectionAddress.chain
        );
     */
    const lockedOnEthLogData1 = parseLogs(
        lockOnEthReceipt1!.logs[1] as EventLog
    );

    const lockedOnEthLogData2 = parseLogs(
        lockOnEthReceipt2!.logs[1] as EventLog
    );

    expect(lockedOnEthLogData1.tokenId).to.be.equal(nftDetails.tokenId1.value);
    expect(lockedOnEthLogData1.destinationChain).to.be.equal(
        bscBridge.chainSymbol
    );
    expect(lockedOnEthLogData1.destinationUserAddress).to.be.equal(
        bscUser.address
    );
    expect(lockedOnEthLogData1.sourceNftContractAddress).to.be.equal(
        nftDetails.collectionAddress
    );
    expect(lockedOnEthLogData1.tokenAmount).to.be.equal(1);
    expect(lockedOnEthLogData1.nftType).to.be.equal(getNftType(nftType));
    expect(lockedOnEthLogData1.sourceChain).to.be.equal(bscBridge.chainSymbol);

    // ---

    expect(lockedOnEthLogData2.tokenId).to.be.equal(nftDetails.tokenId2.value);
    expect(lockedOnEthLogData2.destinationChain).to.be.equal(
        bscBridge.chainSymbol
    );
    expect(lockedOnEthLogData2.destinationUserAddress).to.be.equal(
        bscUser.address
    );
    expect(lockedOnEthLogData2.sourceNftContractAddress).to.be.equal(
        nftDetails.collectionAddress
    );
    expect(lockedOnEthLogData2.tokenAmount).to.be.equal(1);
    expect(lockedOnEthLogData2.nftType).to.be.equal(getNftType(nftType));
    expect(lockedOnEthLogData2.sourceChain).to.be.equal(bscBridge.chainSymbol);

    return [
        lockedOnEthLogData1,
        lockedOnEthLogData2,
        lockOnEthReceipt1,
        lockOnEthReceipt2,
    ];
}

export async function claimOnBSC(
    lockedOnEthLogData1: TProcessedLogs,
    lockedOnEthLogData2: TProcessedLogs,
    lockOnEthReceipt1: ContractTransactionReceipt | null,
    lockOnEthReceipt2: ContractTransactionReceipt | null,
    mintedCollectionOnBSC: ERC721Royalty | ERC1155Royalty,
    mintedCollectionOnBSCAddress: string,
    nftDetails: TNFTDetails,
    bscUser: HardhatEthersSigner,
    ethUser: HardhatEthersSigner,
    bscBridge: TBridge,
    nftType: TNFTType,
    getValidatorSignatures: TGetValidatorSignatures
) {
    const [claimDataArgs1, dataHash1] = createHash(
        lockedOnEthLogData1,
        lockOnEthReceipt1?.hash,
        nftDetails,
        ethUser.address
    );

    const [claimDataArgs2, dataHash2] = createHash(
        lockedOnEthLogData2,
        lockOnEthReceipt2?.hash,
        nftDetails,
        ethUser.address
    );

    const signatures = await Promise.all(
        [dataHash1, dataHash2].map((hash) =>
            getValidatorSignatures(hash, "bsc")
        )
    );

    // ensure that storage is owner of the nft
    const [originalStorage721a, originalStorage721b] = await Promise.all([
        bscBridge.bridge[
            nftType === 721
                ? "originalStorageMapping721"
                : "originalStorageMapping1155"
        ](mintedCollectionOnBSCAddress, bscBridge.chainSymbol),
        bscBridge.bridge[
            nftType === 721
                ? "originalStorageMapping721"
                : "originalStorageMapping1155"
        ](mintedCollectionOnBSCAddress, bscBridge.chainSymbol),
    ]);

    if (nftType === 721) {
        let [owner1, owner2] = await Promise.all([
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                lockedOnEthLogData1.tokenId
            ),
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                lockedOnEthLogData2.tokenId
            ),
        ]);
        expect(owner1).to.be.equal(originalStorage721a);
        expect(owner2).to.be.equal(originalStorage721b);
    } else {
        const [balance1, balance2] = await Promise.all([
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                originalStorage721a,
                nftDetails.tokenId1
            ),
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                originalStorage721b,
                nftDetails.tokenId2
            ),
        ]);
        // Check if storageAddressForCollection has at least one of each token
        expect(balance1).to.be.gt(0);
        expect(balance2).to.be.gt(0);
    }

    await Promise.all(
        [claimDataArgs1, claimDataArgs2].map(async (args, i) => {
            if (nftType === 721) {
                return bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT721(args, signatures[i], {
                        value: FEE.value,
                    })
                    .then((r) => r.wait());
            } else {
                return bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT1155(args, signatures[i], {
                        value: FEE.value,
                    })
                    .then((r) => r.wait());
            }
        })
    );

    // ensure that bsc user is the owner after claiming
    if (nftType === 721) {
        let [owner1, owner2] = await Promise.all([
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                lockedOnEthLogData1.tokenId
            ),
            (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
                lockedOnEthLogData2.tokenId
            ),
        ]);
        expect(owner1).to.be.equal(bscUser.address);
        expect(owner2).to.be.equal(bscUser.address);
    } else {
        const [balance1, balance2] = await Promise.all([
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                bscUser.address,
                nftDetails.tokenId1
            ),
            (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
                bscUser.address,
                nftDetails.tokenId2
            ),
        ]);
        // Check if storageAddressForCollection has at least one of each token
        expect(balance1).to.be.gt(0);
        expect(balance2).to.be.gt(0);
    }
}

type TLockNewArgs = {
    lockedEventDatas: TProcessedLogs[];
    duplicateCollectionContracts: ERC1155Royalty[] | ERC721Royalty[];
    duplicateCollectionAddresses: string[];
    nftDetails: TNFTDetails;
    bscUser: HardhatEthersSigner;
    ethUser: HardhatEthersSigner;
    bscBridge: TBridge;
    ethBridge: TBridge;
    nftType: TNFTType;
};

export async function lock_NEW({
    lockedEventDatas,
    duplicateCollectionContracts,
    duplicateCollectionAddresses,
    nftDetails,
    bscUser,
    ethUser,
    bscBridge,
    ethBridge,
    nftType,
}: TLockNewArgs): Promise<TLockReturn_NEW> {
    const [duplicateCollectionAddress1, duplicateCollectionAddress2] =
        duplicateCollectionAddresses;

    console.log({ bscBridge: bscBridge.chainSymbol }); // MOONBEAM
    console.log({ ethBridge: ethBridge.chainSymbol }); // ARBI

    await Promise.all(
        lockedEventDatas.map(async (data, i) => {
            if (nftType === 721) {
                return (duplicateCollectionContracts[i] as ERC721Royalty)
                    .connect(bscUser)
                    .approve(bscBridge.address, data.tokenId)
                    .then((r) => r.wait());
            } else {
                return (duplicateCollectionContracts[i] as ERC1155Royalty)
                    .connect(bscUser)
                    .setApprovalForAll(bscBridge.address, true)
                    .then((r) => r.wait());
            }
        })
    );

    const [lockOnEthReceipt1, lockOnEthReceipt2] = await Promise.all(
        lockedEventDatas.map(async (data, i) => {
            if (nftType === 721) {
                return bscBridge.bridge
                    .connect(bscUser)
                    .lock721(
                        data.tokenId,
                        ethBridge.chainSymbol,
                        ethUser.address,
                        duplicateCollectionContracts[i]
                    )
                    .then((r) => r.wait());
            } else {
                return bscBridge.bridge
                    .connect(bscUser)
                    .lock1155(
                        data.tokenId,
                        ethBridge.chainSymbol,
                        ethUser.address,
                        duplicateCollectionContracts[i],
                        AMOUNT_TO_LOCK
                    )
                    .then((r) => r.wait());
            }
        })
    );

    const originalStorageAddressForDuplicateCollectionProm1 = bscBridge.bridge[
        nftType === 721
            ? "originalStorageMapping721"
            : "originalStorageMapping1155"
    ](duplicateCollectionAddress1, bscBridge.chainSymbol);

    const duplicateStorageAddressForDuplicateCollectionProm1 = bscBridge.bridge[
        nftType === 721
            ? "duplicateStorageMapping721"
            : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress1, bscBridge.chainSymbol);

    const originalStorageAddressForDuplicateCollectionProm2 = bscBridge.bridge[
        nftType === 721
            ? "originalStorageMapping721"
            : "originalStorageMapping1155"
    ](duplicateCollectionAddress2, bscBridge.chainSymbol);

    const duplicateStorageAddressForDuplicateCollectionProm2 = bscBridge.bridge[
        nftType === 721
            ? "duplicateStorageMapping721"
            : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress2, bscBridge.chainSymbol);

    const [
        originalStorageAddressForDuplicateCollection1,
        duplicateStorageAddressForDuplicateCollection1,
        originalStorageAddressForDuplicateCollection2,
        duplicateStorageAddressForDuplicateCollection2,
    ] = await Promise.all([
        originalStorageAddressForDuplicateCollectionProm1,
        duplicateStorageAddressForDuplicateCollectionProm1,
        originalStorageAddressForDuplicateCollectionProm2,
        duplicateStorageAddressForDuplicateCollectionProm2,
    ]);

    // ======================= LOCK ON ETH - VERIFY ===================

    expect(originalStorageAddressForDuplicateCollection1).to.be.equal(
        ZeroAddress
    );
    expect(duplicateStorageAddressForDuplicateCollection1).to.not.be.equal(
        ZeroAddress
    );

    expect(originalStorageAddressForDuplicateCollection2).to.be.equal(
        ZeroAddress
    );
    expect(duplicateStorageAddressForDuplicateCollection2).to.not.be.equal(
        ZeroAddress
    );

    /**
     *  emit Locked(
            tokenId,
            destinationChain,
            destinationUserAddress,
            address(originalCollectionAddress.contractAddress),
            1,
            TYPEERC721,
            originalCollectionAddress.chain
        );
     */
    const lockedOnEthLogData1 = parseLogs(
        lockOnEthReceipt1!.logs[1] as EventLog
    );

    console.log({ lockedOnEthLogData1 });

    const lockedOnEthLogData2 = parseLogs(
        lockOnEthReceipt2!.logs[1] as EventLog
    );

    expect(lockedOnEthLogData1.tokenId).to.be.equal(nftDetails.tokenId1.value);
    expect(lockedOnEthLogData1.destinationChain).to.be.equal(
        ethBridge.chainSymbol
    );
    expect(lockedOnEthLogData1.destinationUserAddress).to.be.equal(
        ethUser.address
    );
    expect(lockedOnEthLogData1.sourceNftContractAddress).to.be.equal(
        nftDetails.collectionAddress
    );
    expect(lockedOnEthLogData1.tokenAmount).to.be.equal(1);
    expect(lockedOnEthLogData1.nftType).to.be.equal(getNftType(nftType));

    // TODO
    // expect(lockedOnEthLogData1.sourceChain).to.be.equal(ethBridge.chainSymbol);

    // ---

    expect(lockedOnEthLogData2.tokenId).to.be.equal(nftDetails.tokenId2.value);
    expect(lockedOnEthLogData2.destinationChain).to.be.equal(
        ethBridge.chainSymbol
    );
    expect(lockedOnEthLogData2.destinationUserAddress).to.be.equal(
        ethUser.address
    );
    expect(lockedOnEthLogData2.sourceNftContractAddress).to.be.equal(
        nftDetails.collectionAddress
    );
    expect(lockedOnEthLogData2.tokenAmount).to.be.equal(1);
    expect(lockedOnEthLogData2.nftType).to.be.equal(getNftType(nftType));

    // TODO
    // expect(lockedOnEthLogData2.sourceChain).to.be.equal(ethBridge.chainSymbol);

    const parsedLogs1 = parseLogs(lockOnEthReceipt1?.logs[1] as EventLog);
    const parsedLogs2 = parseLogs(lockOnEthReceipt2?.logs[1] as EventLog);

    return [[parsedLogs1, parsedLogs2], lockOnEthReceipt1, lockOnEthReceipt2];
}

type TClaimNewArgs = {
    lockedOnEthLogData1: TProcessedLogs;
    lockedOnEthLogData2: TProcessedLogs;
    lockOnEthReceipt1: ContractTransactionReceipt | null;
    lockOnEthReceipt2: ContractTransactionReceipt | null;
    mintedCollectionOnBSC: ERC721Royalty | ERC1155Royalty;
    mintedCollectionOnBSCAddress: string;
    nftDetails: TNFTDetails;
    bscUser: HardhatEthersSigner;
    ethUser: HardhatEthersSigner;
    bscBridge: TBridge;
    nftType: TNFTType;
    getValidatorSignatures: TGetValidatorSignatures;
};

export async function claim_NEW({
    lockedOnEthLogData1,
    lockedOnEthLogData2,
    lockOnEthReceipt1,
    lockOnEthReceipt2,
    mintedCollectionOnBSC,
    mintedCollectionOnBSCAddress,
    nftDetails,
    bscUser,
    ethUser,
    bscBridge,
    nftType,
    getValidatorSignatures,
}: TClaimNewArgs): Promise<[string[], Contract[]]> {
    console.log("here");

    console.log({
        bscBridge: bscBridge.chainSymbol,
    });

    console.log({ lockedOnEthLogData1 });
    const [claimDataArgs1, dataHash1] = createHash(
        lockedOnEthLogData1,
        lockOnEthReceipt1?.hash,
        nftDetails,
        ethUser.address
    );
    console.log("here1");

    const [claimDataArgs2, dataHash2] = createHash(
        lockedOnEthLogData2,
        lockOnEthReceipt2?.hash,
        nftDetails,
        ethUser.address
    );
    console.log("here2");

    const signatures = await Promise.all(
        [dataHash1, dataHash2].map((hash) =>
            getValidatorSignatures(hash, "bsc")
        )
    );
    console.log("here3");

    // // ensure that storage is owner of the nft
    // const [originalStorage721a, originalStorage721b] = await Promise.all([
    //     bscBridge.bridge[
    //         nftType === 721
    //             ? "originalStorageMapping721"
    //             : "originalStorageMapping1155"
    //     ](mintedCollectionOnBSCAddress, bscBridge.chainSymbol),
    //     bscBridge.bridge[
    //         nftType === 721
    //             ? "originalStorageMapping721"
    //             : "originalStorageMapping1155"
    //     ](mintedCollectionOnBSCAddress, bscBridge.chainSymbol),
    // ]);

    console.log("here4");
    // if (nftType === 721) {
    //     let [owner1, owner2] = await Promise.all([
    //         (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
    //             lockedOnEthLogData1.tokenId
    //         ),
    //         (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
    //             lockedOnEthLogData2.tokenId
    //         ),
    //     ]);
    //     expect(owner1).to.be.equal(originalStorage721a);
    //     expect(owner2).to.be.equal(originalStorage721b);
    // } else {
    //     const [balance1, balance2] = await Promise.all([
    //         (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
    //             originalStorage721a,
    //             nftDetails.tokenId1
    //         ),
    //         (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
    //             originalStorage721b,
    //             nftDetails.tokenId2
    //         ),
    //     ]);
    //     // Check if storageAddressForCollection has at least one of each token
    //     expect(balance1).to.be.gt(0);
    //     expect(balance2).to.be.gt(0);
    // }

    console.log({ claimDataArgs1 });

    await Promise.all(
        [claimDataArgs1, claimDataArgs2].map(async (args, i) => {
            if (nftType === 721) {
                return bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT721(args, signatures[i], {
                        value: FEE.value,
                    })
                    .then((r) => r.wait());
            } else {
                return bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT1155(args, signatures[i], {
                        value: FEE.value,
                    })
                    .then((r) => r.wait());
            }
        })
    );

    const [
        [destinationChainId1, duplicateCollectionAddress1],
        [destinationChainId2, duplicateCollectionAddress2],
    ] = await Promise.all(
        [lockedOnEthLogData1, lockedOnEthLogData2].map((d) =>
            bscBridge.bridge.originalToDuplicateMapping(
                d.sourceNftContractAddress,
                d.sourceChain
            )
        )
    );
    console.log({ duplicateCollectionAddress1, duplicateCollectionAddress2 });
    const duplicateCollectionAddresses = [
        duplicateCollectionAddress1,
        duplicateCollectionAddress2,
    ];

    const duplicateCollectionContracts = await Promise.all(
        duplicateCollectionAddresses.map((contractAddress) =>
            hardhatEthers.getContractAt(
                nftType === 721 ? "ERC721Royalty" : "ERC1155Royalty",
                contractAddress
            )
        )
    );
    return [duplicateCollectionAddresses, duplicateCollectionContracts];

    // ensure that bsc user is the owner after claiming
    // if (nftType === 721) {
    //     let [owner1, owner2] = await Promise.all([
    //         (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
    //             lockedOnEthLogData1.tokenId
    //         ),
    //         (mintedCollectionOnBSC as ERC721Royalty).ownerOf(
    //             lockedOnEthLogData2.tokenId
    //         ),
    //     ]);
    //     expect(owner1).to.be.equal(bscUser.address);
    //     expect(owner2).to.be.equal(bscUser.address);
    // } else {
    //     const [balance1, balance2] = await Promise.all([
    //         (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
    //             bscUser.address,
    //             nftDetails.tokenId1
    //         ),
    //         (mintedCollectionOnBSC as ERC1155Royalty).balanceOf(
    //             bscUser.address,
    //             nftDetails.tokenId2
    //         ),
    //     ]);
    //     // Check if storageAddressForCollection has at least one of each token
    //     expect(balance1).to.be.gt(0);
    //     expect(balance2).to.be.gt(0);
    // }
}
export async function lockOnBSCAndClaimOnEth({
    mintedCollectionOnBSC,
    tokenIds,
    mintedCollectionOnBSCAddress,
    nftDetails,
    bscUser,
    ethUser,
    bscBridge,
    ethBridge,
    nftType,
    getValidatorSignatures,
}: TLockOnBSCAndClaimOnEthArgs) {
    const [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2] =
        await lockOnBSC(
            mintedCollectionOnBSC,
            tokenIds,
            mintedCollectionOnBSCAddress,
            nftDetails,
            bscUser,
            ethUser,
            bscBridge,
            ethBridge,
            nftType
        );
    return await claimOnEth(
        parsedLogs1,
        parsedLogs2,
        lockedReceipt1,
        lockedReceipt2,
        nftDetails,
        ethUser,
        ethBridge,
        nftType,
        getValidatorSignatures
    );
}

export async function lockOnEthAndClaimOnBSC({
    lockedEventDatas,
    duplicateCollectionContracts,
    duplicateCollectionAddresses,
    mintedCollectionOnBSC,
    mintedCollectionOnBSCAddress,
    nftDetails,
    bscUser,
    ethUser,
    bscBridge,
    ethBridge,
    getValidatorSignatures,
    nftType,
}: TLockOnEthAndClaimOnBSCArgs) {
    const [
        lockedOnEthLogData1,
        lockedOnEthLogData2,
        lockOnEthReceipt1,
        lockOnEthReceipt2,
    ] = await lockOnEth(
        lockedEventDatas,
        duplicateCollectionContracts as any,
        duplicateCollectionAddresses,
        nftDetails,
        bscUser,
        ethUser,
        bscBridge,
        ethBridge,
        nftType
    );

    await claimOnBSC(
        lockedOnEthLogData1,
        lockedOnEthLogData2,
        lockOnEthReceipt1,
        lockOnEthReceipt2,
        mintedCollectionOnBSC,
        mintedCollectionOnBSCAddress,
        nftDetails,
        bscUser,
        ethUser,
        bscBridge,
        nftType,
        getValidatorSignatures
    );
}
