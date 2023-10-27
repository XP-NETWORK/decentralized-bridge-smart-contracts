import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import {
    Contract,
    ContractTransactionReceipt,
    EventLog,
    Typed,
    ZeroAddress,
} from "ethers";
import { ethers } from "hardhat";
import {
    Bridge,
    Bridge__factory,
    ERC1155Royalty,
    ERC721Royalty,
} from "../contractsTypes";
import {
    TLockOnBSCAndClaimOnEthReturn,
    TLockReturn,
    TNFTType,
    TProcessedLogs,
} from "./types";
import { Fee, createHash, parseLogs } from "./utils";

type TContractInstance = Bridge;
const AMOUNT_TODO = 1;
type TBridge = {
    bridge: TContractInstance;
    address: string;
    chainSymbol: string;
    collectionDeployer: string;
    storageDeployer: string;
};
describe("Bridge", function () {
    let Bridge: Bridge__factory, bscBridge: TBridge, ethBridge: TBridge;

    let bscValidator1: HardhatEthersSigner,
        bscValidator2: HardhatEthersSigner,
        ethValidator1: HardhatEthersSigner,
        ethValidator2: HardhatEthersSigner,
        bscUser: HardhatEthersSigner,
        ethUser: HardhatEthersSigner,
        ethBridgeDeployer: HardhatEthersSigner,
        bscContractDeployer: HardhatEthersSigner,
        addrs: HardhatEthersSigner[];

    let bscValidators: string[];

    async function deployBridge(
        chainSymbol: string,
        validators: [string, string],
        deployer: HardhatEthersSigner
    ) {
        const CollectionDeployer = await ethers.getContractFactory(
            "NFTCollectionDeployer"
        );
        const collectionInstance = await CollectionDeployer.connect(
            deployer
        ).deploy();
        const collectionDeployer = await collectionInstance
            .connect(deployer)
            .getAddress();

        const StorageDeployer = await ethers.getContractFactory(
            "NFTStorageDeployer"
        );
        const storageDeployerInstance = await StorageDeployer.connect(
            deployer
        ).deploy();
        const storageDeployer = await storageDeployerInstance
            .connect(deployer)
            .getAddress();

        Bridge = await ethers.getContractFactory("Bridge");

        const bridge = await Bridge.connect(deployer).deploy(
            validators,
            chainSymbol,
            collectionDeployer,
            storageDeployer
        );
        const address = await bridge.getAddress();
        return {
            address,
            bridge,
            chainSymbol,
            collectionDeployer,
            storageDeployer,
        };
    }

    beforeEach(async function () {
        [
            bscContractDeployer,
            bscValidator1,
            bscValidator2,
            ethValidator1,
            ethValidator2,
            bscUser,
            ethBridgeDeployer,
            ethUser,
            ...addrs
        ] = await ethers.getSigners();

        bscValidators = [bscValidator1.address, bscValidator2.address];

        bscBridge = await deployBridge(
            "BSC",
            [bscValidator1.address, bscValidator2.address],
            bscContractDeployer
        );

        ethBridge = await deployBridge(
            "ETH",
            [ethValidator1.address, ethValidator2.address],
            ethBridgeDeployer
        );
    });

    describe("Deployment", function () {
        it("Should set the correct Collection Deployer address", async function () {
            expect(await bscBridge.bridge.collectionDeployer()).to.equal(
                bscBridge.collectionDeployer
            );
        });

        it("Should set the correct Storage Deployer address", async function () {
            expect(await bscBridge.bridge.storageDeployer()).to.equal(
                bscBridge.storageDeployer
            );
        });

        it("Should set validators correctly", async function () {
            for (let validator of bscValidators) {
                expect(await bscBridge.bridge.validators(validator)).to.equal(
                    true
                );
            }
        });

        it("Should set the correct Chain Symbol", async function () {
            expect(await bscBridge.bridge.selfChain()).to.be.equal("BSC");
        });

        it("Should fail if Collection Deployer address OR Storage Deployer address is address zero", async function () {
            expect(await bscBridge.bridge.collectionDeployer()).to.not.be.equal(
                ethers.ZeroAddress
            );
            expect(await bscBridge.bridge.storageDeployer()).to.not.be.equal(
                ethers.ZeroAddress
            );
        });

        it("Should fail to initialize contract if collection or storage address is zero", async function () {
            await expect(
                Bridge.deploy(
                    bscValidators,
                    bscBridge.chainSymbol,
                    ethers.ZeroAddress,
                    ethers.ZeroAddress
                )
            ).to.be.rejected;
        });

        it("Should have the correct validators count", async function () {
            expect(await bscBridge.bridge.validatorsCount()).to.be.equal(
                bscValidators.length
            );
        });
    });

    describe("721 NFT Integration Test", function () {
        const uint256 = ethers.Typed.uint256;

        let nftDetails = {
            toAddress: "",
            tokenId1: uint256(0),
            tokenId2: uint256(0),
            royalty: uint256(0),
            royaltyReceiver: "",
            tokenURI: "",
            collectionAddress: "",
            name: "",
            symbol: "",
        };

        let mintedCollectionOnBSC: ERC721Royalty;
        let mintedCollectionOnBSCAddress: string;
        let tokenIds: Typed[] = [];

        beforeEach(async function () {
            const CollectionDeployer = await ethers.getContractFactory(
                "NFTCollectionDeployer"
            );

            const collectionInstance = await CollectionDeployer.connect(
                bscUser
            ).deploy();
            collectionInstance.setOwner(bscUser.address);

            const name = "MyCollection";
            const symbol = "MC";
            const toWait = await collectionInstance.deployNFT721Collection(
                name,
                symbol
            );
            const response = await toWait.wait();

            if (!response) return;

            const logs = response.logs[1] as EventLog;
            const newCollectionAddress = logs.args[0];

            mintedCollectionOnBSC = await ethers.getContractAt(
                "ERC721Royalty",
                newCollectionAddress
            );

            mintedCollectionOnBSCAddress =
                await mintedCollectionOnBSC.getAddress();

            const toAddress = bscUser.address;
            const tokenId1 = ethers.Typed.uint256(1);
            const tokenId2 = ethers.Typed.uint256(2);
            tokenIds = [tokenId1, tokenId2];
            const royalty = ethers.Typed.uint256(100);
            const royaltyReceiver = bscUser.address;
            const tokenURI = "";

            nftDetails = {
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
                const mintArgs: Parameters<typeof mintedCollectionOnBSC.mint> =
                    [toAddress, id, royalty, royaltyReceiver, tokenURI];
                return mintedCollectionOnBSC.connect(bscUser).mint(...mintArgs);
            });

            await Promise.all(mintPromises);
        });
        async function lockOnBSC(
            mintedCollectionOnBSC: ERC1155Royalty | ERC721Royalty,
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
                                AMOUNT_TODO
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
            expect(storageAddressForCollection).to.not.be.equal(
                ethers.ZeroAddress
            );

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
            expect(parsedLogs1.destinationChain).to.be.equal(
                ethBridge.chainSymbol
            );
            expect(parsedLogs1.destinationUserAddress).to.be.equal(
                ethUser.address
            );
            expect(parsedLogs1.sourceNftContractAddress).to.be.equal(
                mintedCollectionOnBSCAddress
            );
            expect(parsedLogs1.tokenAmount).to.be.equal(1);
            expect(parsedLogs1.nftType).to.be.equal("singular");
            expect(parsedLogs1.sourceChain).to.be.equal(bscBridge.chainSymbol);

            expect(parsedLogs2.tokenId).to.be.equal(nftDetails.tokenId2.value);
            expect(parsedLogs2.destinationChain).to.be.equal(
                ethBridge.chainSymbol
            );
            expect(parsedLogs2.destinationUserAddress).to.be.equal(
                ethUser.address
            );
            expect(parsedLogs2.sourceNftContractAddress).to.be.equal(
                mintedCollectionOnBSCAddress
            );
            expect(parsedLogs2.tokenAmount).to.be.equal(1);
            expect(parsedLogs2.nftType).to.be.equal(
                nftType === 721 ? "singular" : "multiple"
            );
            expect(parsedLogs2.sourceChain).to.be.equal(bscBridge.chainSymbol);

            return [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2];
        }

        async function claimOnEth(
            parsedLogs1: TProcessedLogs,
            parsedLogs2: TProcessedLogs,
            lockedReceipt1: ContractTransactionReceipt | null,
            lockedReceipt2: ContractTransactionReceipt | null,
            nftType: TNFTType
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

                    const signatures = await getValidatorSignatures(
                        hash,
                        "eth"
                    );

                    return ethBridge.bridge
                        .connect(ethUser)
                        [nftType === 721 ? "claimNFT721" : "claimNFT1155"](
                            data,
                            signatures,
                            {
                                value: Fee.value,
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
                    ethers.getContractAt(
                        nftType === 721 ? "ERC721Royalty" : "ERC1155Royalty",
                        contractAddress
                    )
                )
            );

            const [duplicateCollectionContract1, duplicateCollectionContract2] =
                duplicateCollectionContracts as unknown as
                    | ERC721Royalty[]
                    | ERC1155Royalty[];

            // TODO: need to change ways in which I am checking the owner
            if (nftType === 721) {
                const duplicateNFTOwnerProm1 = (
                    duplicateCollectionContract1 as ERC721Royalty
                ).ownerOf(ethers.Typed.uint256(parsedLogs1.tokenId));

                const duplicateNFTOwnerProm2 = (
                    duplicateCollectionContract2 as ERC721Royalty
                ).ownerOf(ethers.Typed.uint256(parsedLogs2.tokenId));

                const royaltyInfoProm1 =
                    duplicateCollectionContract1.royaltyInfo(
                        ethers.Typed.uint256(parsedLogs1.tokenId),
                        ethers.Typed.uint256(1)
                    );

                const royaltyInfoProm2 =
                    duplicateCollectionContract2.royaltyInfo(
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
                ).balanceOf(
                    ethUser.address,
                    ethers.Typed.uint256(parsedLogs1.tokenId)
                );

                const duplicateNFTOwnerProm2 = (
                    duplicateCollectionContract2 as ERC1155Royalty
                ).balanceOf(
                    ethUser.address,
                    ethers.Typed.uint256(parsedLogs2.tokenId)
                );

                const royaltyInfoProm1 =
                    duplicateCollectionContract1.royaltyInfo(
                        ethers.Typed.uint256(parsedLogs1.tokenId),
                        ethers.Typed.uint256(1)
                    );

                const royaltyInfoProm2 =
                    duplicateCollectionContract2.royaltyInfo(
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

        async function lockOnEth(
            lockedEventDatas: TProcessedLogs[],
            duplicateCollectionContracts: ERC1155Royalty[] | ERC721Royalty[],
            duplicateCollectionAddresses: string[],
            nftType: TNFTType
        ): Promise<TLockReturn> {
            const [duplicateCollectionAddress1, duplicateCollectionAddress2] =
                duplicateCollectionAddresses;

            await Promise.all(
                lockedEventDatas.map(async (data, i) => {
                    if (nftType === 721) {
                        return (
                            duplicateCollectionContracts[i] as ERC721Royalty
                        )
                            .connect(ethUser)
                            .approve(ethBridge.address, data.tokenId)
                            .then((r) => r.wait());
                    } else {
                        return (
                            duplicateCollectionContracts[i] as ERC1155Royalty
                        )
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
                                AMOUNT_TODO
                            )
                            .then((r) => r.wait());
                    }
                })
            );

            const originalStorageAddressForDuplicateCollectionProm1 =
                ethBridge.bridge[
                    nftType === 721
                        ? "originalStorageMapping721"
                        : "originalStorageMapping1155"
                ](duplicateCollectionAddress1, bscBridge.chainSymbol);

            const duplicateStorageAddressForDuplicateCollectionProm1 =
                ethBridge.bridge[
                    nftType === 721
                        ? "duplicateStorageMapping721"
                        : "duplicateStorageMapping1155"
                ](duplicateCollectionAddress1, ethBridge.chainSymbol);

            const originalStorageAddressForDuplicateCollectionProm2 =
                ethBridge.bridge[
                    nftType === 721
                        ? "originalStorageMapping721"
                        : "originalStorageMapping1155"
                ](duplicateCollectionAddress2, bscBridge.chainSymbol);

            const duplicateStorageAddressForDuplicateCollectionProm2 =
                ethBridge.bridge[
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
            expect(
                duplicateStorageAddressForDuplicateCollection1
            ).to.not.be.equal(ZeroAddress);
            expect(originalStorageAddressForDuplicateCollection2).to.be.equal(
                ZeroAddress
            );
            expect(
                duplicateStorageAddressForDuplicateCollection2
            ).to.not.be.equal(ZeroAddress);

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

            expect(lockedOnEthLogData1.tokenId).to.be.equal(
                nftDetails.tokenId1.value
            );
            expect(lockedOnEthLogData1.destinationChain).to.be.equal(
                bscBridge.chainSymbol
            );
            expect(lockedOnEthLogData1.destinationUserAddress).to.be.equal(
                bscUser.address
            );
            expect(lockedOnEthLogData1.sourceNftContractAddress).to.be.equal(
                nftDetails.collectionAddress
            );
            // TODO: make it dynamic
            expect(lockedOnEthLogData1.tokenAmount).to.be.equal(1);
            expect(lockedOnEthLogData1.nftType).to.be.equal(
                nftType === 721 ? "singular" : "multiple"
            );
            expect(lockedOnEthLogData1.sourceChain).to.be.equal(
                bscBridge.chainSymbol
            );

            // ---

            expect(lockedOnEthLogData2.tokenId).to.be.equal(
                nftDetails.tokenId2.value
            );
            expect(lockedOnEthLogData2.destinationChain).to.be.equal(
                bscBridge.chainSymbol
            );
            expect(lockedOnEthLogData2.destinationUserAddress).to.be.equal(
                bscUser.address
            );
            expect(lockedOnEthLogData2.sourceNftContractAddress).to.be.equal(
                nftDetails.collectionAddress
            );
            // TODO: make it dynamic
            expect(lockedOnEthLogData2.tokenAmount).to.be.equal(1);
            expect(lockedOnEthLogData2.nftType).to.be.equal(
                nftType === 721 ? "singular" : "multiple"
            );
            expect(lockedOnEthLogData2.sourceChain).to.be.equal(
                bscBridge.chainSymbol
            );

            return [
                lockedOnEthLogData1,
                lockedOnEthLogData2,
                lockOnEthReceipt1,
                lockOnEthReceipt2,
            ];
        }

        const getValidatorSignatures = async (
            hash: Uint8Array,
            type: "eth" | "bsc"
        ): Promise<[string, string]> => {
            let promises: [Promise<string>, Promise<string>];
            switch (type) {
                case "eth":
                    promises = [
                        ethValidator1.signMessage(hash),
                        ethValidator2.signMessage(hash),
                    ];
                    break;
                case "bsc":
                    promises = [
                        bscValidator1.signMessage(hash),
                        bscValidator2.signMessage(hash),
                    ];
                    break;
                default:
                    promises = [
                        ethValidator1.signMessage(hash),
                        ethValidator2.signMessage(hash),
                    ];
                    break;
            }

            return await Promise.all(promises);
        };

        async function claimOnBSC(
            lockedOnEthLogData1: TProcessedLogs,
            lockedOnEthLogData2: TProcessedLogs,
            lockOnEthReceipt1: ContractTransactionReceipt | null,
            lockOnEthReceipt2: ContractTransactionReceipt | null,
            mintedCollectionOnBSC: ERC721Royalty | ERC1155Royalty,
            nftType: TNFTType
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
            const [originalStorage721a, originalStorage721b] =
                await Promise.all([
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

            // TODO: extract to a function
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
                [claimDataArgs1, claimDataArgs2].map((args, i) => {
                    if (nftType === 721) {
                        return bscBridge.bridge
                            .connect(bscUser)
                            .claimNFT721(args, signatures[i], {
                                value: Fee.value,
                            })
                            .then((r) => r.wait());
                    } else {
                        return bscBridge.bridge
                            .connect(bscUser)
                            .claimNFT1155(args, signatures[i], {
                                value: Fee.value,
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

        async function lockOnBSCAndClaimOnEth(): TLockOnBSCAndClaimOnEthReturn {
            const nftType = 721;
            const [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2] =
                await lockOnBSC(mintedCollectionOnBSC, nftType);

            return await claimOnEth(
                parsedLogs1,
                parsedLogs2,
                lockedReceipt1,
                lockedReceipt2,
                nftType
            );
        }

        async function lockOnEthAndClaimOnBSC(
            lockedEventDatas: TProcessedLogs[],
            duplicateCollectionContracts: Contract[],
            duplicateCollectionAddresses: string[]
        ) {
            const nftType = 721;

            const [
                lockedOnEthLogData1,
                lockedOnEthLogData2,
                lockOnEthReceipt1,
                lockOnEthReceipt2,
            ] = await lockOnEth(
                lockedEventDatas,
                duplicateCollectionContracts as any,
                duplicateCollectionAddresses,
                nftType
            );

            await claimOnBSC(
                lockedOnEthLogData1,
                lockedOnEthLogData2,
                lockOnEthReceipt1,
                lockOnEthReceipt2,
                mintedCollectionOnBSC,
                nftType
            );
        }

        it("Should successfully run the complete flow to and fro with multiple 721 NFT", async function () {
            const CYCLES = 2;

            for (let cycle = 0; cycle < CYCLES; cycle++) {
                const [
                    lockedEventDatas,
                    duplicateCollectionAddresses,
                    duplicateCollectionContracts,
                ] = await lockOnBSCAndClaimOnEth();

                await lockOnEthAndClaimOnBSC(
                    lockedEventDatas,
                    duplicateCollectionContracts,
                    duplicateCollectionAddresses
                );
            }
        });
    });
});
