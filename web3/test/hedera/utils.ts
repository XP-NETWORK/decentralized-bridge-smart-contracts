
import { expect } from "chai";
import {
  Contract,
  ContractTransactionReceipt,
  EventLog,
  Typed,
  Wallet,
  ZeroAddress,
  ethers,
  keccak256,
} from "ethers";
import { ethers as hardhatEthers } from "hardhat";
import { ERC1155Royalty, ERC721Royalty } from "../../contractsTypes";
import {
  THederaBridge,
  TChainArrWithBridge,
  TCreateHashReturn,
  TGetValidatorSignatures,
  TLockOnBSCAndClaimOnEthArgs,
  TLockOnBSCAndClaimOnEthReturn,
  TLockOnEthAndClaimOnBSCArgs,
  TLockReturn,
  TLockReturn2,
  TLockedEventData,
  TNFTDetails,
  TNFTType,
  TProcessedLogs,
} from "./types";

export const encoder = new ethers.AbiCoder();
export const FEE = ethers.Typed.uint256(5);
export const AMOUNT_TO_LOCK = 1;

export const NftTransferDetailsTypes = [
  "uint256", // 0 - tokenId
  "string", // 1 - sourceChain
  "string", // 2 - destinationChain
  "address", // 3 - destinationUserAddress
  "string", // 4 - sourceNftContractAddress
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
    sourceNftContractAddress:
      lockedEventData.sourceNftContractAddress.toLowerCase(),
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

export async function deploy721Collection(bscUser: Wallet) {
  const name = "MyCollection";
  const symbol = "MC";

  const CollectionDeployer = await hardhatEthers.getContractFactory(
    "NFTCollectionDeployer"
  );

  const collectionInstance = await CollectionDeployer.connect(bscUser).deploy();

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

  const mintedCollectionOnBSCAddress = await mintedCollectionOnBSC.getAddress();

  const toAddress = bscUser.address;
  const tokenId1 = ethers.Typed.uint256(1);
  const tokenId2 = ethers.Typed.uint256(2);
  const tokenIds: [Typed, Typed] = [tokenId1, tokenId2];
  const royalty = ethers.Typed.uint256(100);
  const royaltyReceiver = bscUser.address;
  const tokenURI = "";

  const nftDetails: TNFTDetails = {
    toAddress,
    tokenId1,
    tokenId2,
    royalty,
    royaltyReceiver,
    tokenURI,
    collectionAddress: newCollectionAddress,
    name,
    symbol,
    chainSymbolOfNFT: "BSC",
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
  bscUser: Wallet
) {
  const name = "MyCollection";
  const symbol = "MC";

  const CollectionDeployer = await hardhatEthers.getContractFactory(
    "NFTCollectionDeployer"
  );

  const collectionInstance = await CollectionDeployer.connect(bscUser).deploy();

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

  const mintedCollectionOnBSCAddress = await mintedCollectionOnBSC.getAddress();

  const toAddress = bscUser.address;
  const tokenId1 = ethers.Typed.uint256(1);
  const tokenId2 = ethers.Typed.uint256(2);
  const tokenIds: [Typed, Typed] = [tokenId1, tokenId2];
  const royalty = ethers.Typed.uint256(100);
  const royaltyReceiver = bscUser.address;
  const tokenURI = "";

  const nftDetails: TNFTDetails = {
    toAddress,
    tokenId1,
    tokenId2,
    royalty,
    royaltyReceiver,
    tokenURI,
    collectionAddress: newCollectionAddress,
    name,
    symbol,
    chainSymbolOfNFT: "BSC",
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
  bscUser: Wallet,
  ethUser: Wallet,
  bscBridge: THederaBridge,
  ethBridge: THederaBridge,
  nftType: TNFTType,
  amountToLock?: number
): Promise<TLockReturn> {
  const toLock = amountToLock === undefined ? AMOUNT_TO_LOCK : amountToLock;

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
      }
    })
  );

  // get the new storage contract address for the original nft
  const storageAddressForCollection = await bscBridge.bridge[
    nftType === 721 ? "originalStorageMapping721" : "originalStorageMapping1155"
  ](nftDetails.collectionAddress.toLowerCase(), "BSC");

  expect(storageAddressForCollection).to.not.be.equal(ethers.ZeroAddress);

  if (nftType === 721) {
    const [owner1, owner2] = await Promise.all([
      (mintedCollectionOnBSC as ERC721Royalty).ownerOf(nftDetails.tokenId1),
      (mintedCollectionOnBSC as ERC721Royalty).ownerOf(nftDetails.tokenId2),
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
  expect(parsedLogs1.sourceNftContractAddress.toLowerCase()).to.be.equal(
    mintedCollectionOnBSCAddress.toLowerCase()
  );
  expect(parsedLogs1.tokenAmount).to.be.equal(toLock);
  expect(parsedLogs1.nftType).to.be.equal(getNftType(nftType));
  expect(parsedLogs1.sourceChain).to.be.equal(bscBridge.chainSymbol);

  expect(parsedLogs2.tokenId).to.be.equal(nftDetails.tokenId2.value);
  expect(parsedLogs2.destinationChain).to.be.equal(ethBridge.chainSymbol);
  expect(parsedLogs2.destinationUserAddress).to.be.equal(ethUser.address);
  expect(parsedLogs2.sourceNftContractAddress.toLowerCase()).to.be.equal(
    mintedCollectionOnBSCAddress.toLowerCase()
  );
  expect(parsedLogs2.tokenAmount).to.be.equal(toLock);
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
  ethUser: Wallet,
  ethBridge: THederaBridge,
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
        [nftType === 721 ? "claimNFT721" : "claimNFT1155"](data, signatures, {
          value: FEE.value,
        })
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

    const [duplicateNFTOwner1, royaltyInfo1, duplicateNFTOwner2, royaltyInfo2] =
      await Promise.all([
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

    const [duplicateNFTOwner1, royaltyInfo1, duplicateNFTOwner2, royaltyInfo2] =
      await Promise.all([
        duplicateNFTOwnerProm1,
        royaltyInfoProm1,
        duplicateNFTOwnerProm2,
        royaltyInfoProm2,
      ]);

    expect(duplicateNFTOwner1).to.be.gt(0);
    expect(royaltyInfo1[1]).to.be.equal(nftDetails.royalty.value); // value
    expect(duplicateNFTOwner2).to.be.gt(0);
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
  bscUser: Wallet,
  ethUser: Wallet,
  bscBridge: THederaBridge,
  ethBridge: THederaBridge,
  nftType: TNFTType,
  amountToLock?: number
): Promise<TLockReturn> {
  const toLock = amountToLock === undefined ? AMOUNT_TO_LOCK : amountToLock;

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
        throw new Error("Function not implemented.");
      }
    })
  );

  const originalStorageAddressForDuplicateCollectionProm1 = ethBridge.bridge[
    nftType === 721 ? "originalStorageMapping721" : "originalStorageMapping1155"
  ](duplicateCollectionAddress1.toLowerCase(), bscBridge.chainSymbol);

  const duplicateStorageAddressForDuplicateCollectionProm1 = ethBridge.bridge[
    nftType === 721
      ? "duplicateStorageMapping721"
      : "duplicateStorageMapping1155"
  ](duplicateCollectionAddress1.toLowerCase(), ethBridge.chainSymbol);

  const originalStorageAddressForDuplicateCollectionProm2 = ethBridge.bridge[
    nftType === 721 ? "originalStorageMapping721" : "originalStorageMapping1155"
  ](duplicateCollectionAddress2.toLowerCase(), bscBridge.chainSymbol);

  const duplicateStorageAddressForDuplicateCollectionProm2 = ethBridge.bridge[
    nftType === 721
      ? "duplicateStorageMapping721"
      : "duplicateStorageMapping1155"
  ](duplicateCollectionAddress2.toLowerCase(), ethBridge.chainSymbol);

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
  const lockedOnEthLogData1 = parseLogs(lockOnEthReceipt1!.logs[1] as EventLog);

  const lockedOnEthLogData2 = parseLogs(lockOnEthReceipt2!.logs[1] as EventLog);

  expect(lockedOnEthLogData1.tokenId).to.be.equal(nftDetails.tokenId1.value);
  expect(lockedOnEthLogData1.destinationChain).to.be.equal(
    bscBridge.chainSymbol
  );
  expect(lockedOnEthLogData1.destinationUserAddress).to.be.equal(
    bscUser.address
  );
  expect(
    lockedOnEthLogData1.sourceNftContractAddress.toLowerCase()
  ).to.be.equal(nftDetails.collectionAddress.toLowerCase());
  expect(lockedOnEthLogData1.tokenAmount).to.be.equal(toLock);
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
  expect(
    lockedOnEthLogData2.sourceNftContractAddress.toLowerCase()
  ).to.be.equal(nftDetails.collectionAddress.toLowerCase());
  expect(lockedOnEthLogData2.tokenAmount).to.be.equal(toLock);
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
  bscUser: Wallet,
  ethUser: Wallet,
  bscBridge: THederaBridge,
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
    [dataHash1, dataHash2].map((hash) => getValidatorSignatures(hash, "bsc"))
  );

  // ensure that storage is owner of the nft
  const [originalStorage721a, originalStorage721b] = await Promise.all([
    bscBridge.bridge[
      nftType === 721
        ? "originalStorageMapping721"
        : "originalStorageMapping1155"
    ](mintedCollectionOnBSCAddress.toLowerCase(), bscBridge.chainSymbol),
    bscBridge.bridge[
      nftType === 721
        ? "originalStorageMapping721"
        : "originalStorageMapping1155"
    ](mintedCollectionOnBSCAddress.toLowerCase(), bscBridge.chainSymbol),
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
        throw new Error("Function not implemented.");
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
  sourceUser: Wallet;
  destinationUser: Wallet;
  source: TChainArrWithBridge;
  destination: TChainArrWithBridge;
  nftType: TNFTType;
};

export async function lock({
  lockedEventDatas,
  duplicateCollectionContracts,
  duplicateCollectionAddresses,
  nftDetails,
  sourceUser,
  destinationUser,
  source,
  destination,
  nftType,
}: TLockNewArgs): Promise<TLockReturn2> {
  const sourceBridge = source.bridge;
  const destinationBridge = destination.bridge;

  const [duplicateCollectionAddress1, duplicateCollectionAddress2] =
    duplicateCollectionAddresses;

  await Promise.all(
    lockedEventDatas.map(async (data, i) => {
      if (nftType === 721) {
        return (duplicateCollectionContracts[i] as ERC721Royalty)
          .connect(sourceUser)
          .approve(sourceBridge.address, data.tokenId)
          .then((r) => r.wait());
      } else {
        return (duplicateCollectionContracts[i] as ERC1155Royalty)
          .connect(sourceUser)
          .setApprovalForAll(sourceBridge.address, true)
          .then((r) => r.wait());
      }
    })
  );

  const [lockOnEthReceipt1, lockOnEthReceipt2] = await Promise.all(
    lockedEventDatas.map(async (data, i) => {
      if (nftType === 721) {
        return sourceBridge.bridge
          .connect(sourceUser)
          .lock721(
            data.tokenId,
            destinationBridge.chainSymbol,
            destinationUser.address,
            duplicateCollectionContracts[i]
          )
          .then((r) => r.wait());
      } else {
        throw new Error("Function not implemented.");
      }
    })
  );

  const originalStorageAddressForDuplicateCollectionProm1 = sourceBridge.bridge[
    nftType === 721 ? "originalStorageMapping721" : "originalStorageMapping1155"
  ](duplicateCollectionAddress1.toLowerCase(), sourceBridge.chainSymbol);

  const duplicateStorageAddressForDuplicateCollectionProm1 =
    sourceBridge.bridge[
      nftType === 721
        ? "duplicateStorageMapping721"
        : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress1.toLowerCase(), sourceBridge.chainSymbol);

  const originalStorageAddressForDuplicateCollectionProm2 = sourceBridge.bridge[
    nftType === 721 ? "originalStorageMapping721" : "originalStorageMapping1155"
  ](duplicateCollectionAddress2.toLowerCase(), sourceBridge.chainSymbol);

  const duplicateStorageAddressForDuplicateCollectionProm2 =
    sourceBridge.bridge[
      nftType === 721
        ? "duplicateStorageMapping721"
        : "duplicateStorageMapping1155"
    ](duplicateCollectionAddress2.toLowerCase(), sourceBridge.chainSymbol);

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
  const lockedOnEthLogData1 = parseLogs(lockOnEthReceipt1!.logs[1] as EventLog);

  const lockedOnEthLogData2 = parseLogs(lockOnEthReceipt2!.logs[1] as EventLog);

  expect(lockedOnEthLogData1.tokenId).to.be.equal(nftDetails.tokenId1.value);
  expect(lockedOnEthLogData1.destinationChain).to.be.equal(
    destinationBridge.chainSymbol
  );
  expect(lockedOnEthLogData1.destinationUserAddress).to.be.equal(
    destinationUser.address
  );
  expect(
    lockedOnEthLogData1.sourceNftContractAddress.toLowerCase()
  ).to.be.equal(nftDetails.collectionAddress.toLowerCase());
  expect(lockedOnEthLogData1.tokenAmount).to.be.equal(1);
  expect(lockedOnEthLogData1.nftType).to.be.equal(getNftType(nftType));

  expect(lockedOnEthLogData1.sourceChain).to.be.equal(
    nftDetails.chainSymbolOfNFT
  );

  // ---

  expect(lockedOnEthLogData2.tokenId).to.be.equal(nftDetails.tokenId2.value);
  expect(lockedOnEthLogData2.destinationChain).to.be.equal(
    destinationBridge.chainSymbol
  );
  expect(lockedOnEthLogData2.destinationUserAddress).to.be.equal(
    destinationUser.address
  );
  expect(
    lockedOnEthLogData2.sourceNftContractAddress.toLowerCase()
  ).to.be.equal(nftDetails.collectionAddress.toLowerCase());
  expect(lockedOnEthLogData2.tokenAmount).to.be.equal(1);
  expect(lockedOnEthLogData2.nftType).to.be.equal(getNftType(nftType));

  expect(lockedOnEthLogData2.sourceChain).to.be.equal(
    nftDetails.chainSymbolOfNFT
  );

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
  destinationUser: Wallet;
  sourceUser: Wallet;
  destinationBridge: THederaBridge;
  nftType: TNFTType;
  getValidatorSignatures: TGetValidatorSignatures;
};

export async function claim({
  lockedOnEthLogData1,
  lockedOnEthLogData2,
  lockOnEthReceipt1,
  lockOnEthReceipt2,
  mintedCollectionOnBSC,
  mintedCollectionOnBSCAddress,
  nftDetails,
  destinationUser,
  sourceUser,
  destinationBridge,
  nftType,
  getValidatorSignatures,
}: TClaimNewArgs): Promise<[string[], Contract[]]> {
  const [claimDataArgs1, dataHash1] = createHash(
    lockedOnEthLogData1,
    lockOnEthReceipt1?.hash,
    nftDetails,
    sourceUser.address
  );

  const [claimDataArgs2, dataHash2] = createHash(
    lockedOnEthLogData2,
    lockOnEthReceipt2?.hash,
    nftDetails,
    sourceUser.address
  );

  const signatures = await Promise.all(
    [dataHash1, dataHash2].map((hash) => getValidatorSignatures(hash, "bsc"))
  );

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

  await Promise.all(
    [claimDataArgs1, claimDataArgs2].map(async (args, i) => {
      if (nftType === 721) {
        return destinationBridge.bridge
          .connect(destinationUser)
          .claimNFT721(args, signatures[i], {
            value: FEE.value,
          })
          .then((r) => r.wait());
      } else {
        throw new Error("Function not implemented.");
      }
    })
  );

  const [[, duplicateCollectionAddress1], [, duplicateCollectionAddress2]] =
    await Promise.all(
      [lockedOnEthLogData1, lockedOnEthLogData2].map((d) =>
        destinationBridge.bridge.originalToDuplicateMapping(
          d.sourceNftContractAddress,
          d.sourceChain
        )
      )
    );
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
  amountToLock,
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
      nftType,
      amountToLock
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
  amountToLock,
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
    nftType,
    amountToLock
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

export function makeBytesLike(signature: string): string {
  return ethers.hexlify(ethers.toUtf8Bytes(signature));
}

export function formatSignatures(
  signatures: [string, string]
): { signature: string; signerAddress: string }[] {
  return signatures.map((sig, i) => {
    return {
      signature: makeBytesLike(sig),
      signerAddress: `signerAddress${i}`,
    };
  });
}
