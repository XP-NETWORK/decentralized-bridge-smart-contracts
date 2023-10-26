import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import {
    ContractTransactionReceipt,
    EventLog,
    Typed,
    ZeroAddress,
} from "ethers";
import { ethers } from "hardhat";
import { Bridge, Bridge__factory, ERC721Royalty } from "../contractsTypes";
import { TLockOnBSCAndClaimOnEthReturn, TProcessedLogs } from "./types";
import { Fee, createHash, parseLogs } from "./utils";

type TContractInstance = Bridge;

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

        async function lockOnBSC(): Promise<
            [
                TProcessedLogs,
                TProcessedLogs,
                ContractTransactionReceipt | null,
                ContractTransactionReceipt | null
            ]
        > {
            await Promise.all(
                tokenIds.map(async (id) => {
                    return mintedCollectionOnBSC
                        .connect(bscUser)
                        .approve(bscBridge.address, id)
                        .then((r) => r.wait());
                })
            );
            /// lock the nft by creating a new storage
            const [lockedReceipt1, lockedReceipt2] = await Promise.all(
                tokenIds.map((id) =>
                    bscBridge.bridge
                        .connect(bscUser)
                        ["lock721"](
                            id,
                            ethBridge.chainSymbol,
                            ethUser.address,
                            nftDetails.collectionAddress
                        )
                        .then((r) => r.wait())
                )
            );

            // get the new storage contract address for the original nft
            const storageAddressForCollection = await bscBridge.bridge[
                "originalStorageMapping721"
            ](nftDetails.collectionAddress, "BSC");
            expect(storageAddressForCollection).to.not.be.equal(
                ethers.ZeroAddress
            );

            const [owner1, owner2] = await Promise.all([
                mintedCollectionOnBSC.ownerOf(nftDetails.tokenId1),
                mintedCollectionOnBSC.ownerOf(nftDetails.tokenId2),
            ]);
            expect(storageAddressForCollection).to.be.equal(owner1);
            expect(storageAddressForCollection).to.be.equal(owner2);

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
            expect(parsedLogs2.nftType).to.be.equal("singular");
            expect(parsedLogs2.sourceChain).to.be.equal(bscBridge.chainSymbol);

            return [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2];
        }

        async function claimOnEth(
            parsedLogs1: TProcessedLogs,
            parsedLogs2: TProcessedLogs,
            lockedReceipt1: ContractTransactionReceipt | null,
            lockedReceipt2: ContractTransactionReceipt | null
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
                        ["claimNFT721"](data, signatures, {
                            value: Fee.value,
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
                    ethers.getContractAt(
                        // TODO: Make this dynamic as well
                        "ERC721Royalty",
                        contractAddress
                    )
                )
            );

            const [duplicateCollectionContract1, duplicateCollectionContract2] =
                duplicateCollectionContracts;

            const duplicateNFTOwnerProm1 = duplicateCollectionContract1.ownerOf(
                ethers.Typed.uint256(parsedLogs1.tokenId)
            );

            const royaltyInfoProm1 = duplicateCollectionContract1.royaltyInfo(
                ethers.Typed.uint256(parsedLogs1.tokenId),
                ethers.Typed.uint256(1)
            );

            const tokenURIProm1 = duplicateCollectionContract1.tokenURI(
                ethers.Typed.uint256(parsedLogs1.tokenId)
            );

            const duplicateNFTOwnerProm2 = duplicateCollectionContract2.ownerOf(
                ethers.Typed.uint256(parsedLogs2.tokenId)
            );

            const royaltyInfoProm2 = duplicateCollectionContract2.royaltyInfo(
                ethers.Typed.uint256(parsedLogs2.tokenId),
                ethers.Typed.uint256(1)
            );

            const tokenURIProm3 = duplicateCollectionContract2.tokenURI(
                ethers.Typed.uint256(parsedLogs2.tokenId)
            );

            const [
                duplicateNFTOwner1,
                royaltyInfo1,
                tokenURI1,
                duplicateNFTOwner2,
                royaltyInfo2,
                tokenURI2,
            ] = await Promise.all([
                duplicateNFTOwnerProm1,
                royaltyInfoProm1,
                tokenURIProm1,
                duplicateNFTOwnerProm2,
                royaltyInfoProm2,
                tokenURIProm3,
            ]);

            expect(duplicateNFTOwner1).to.be.equal(ethUser.address);
            expect(royaltyInfo1[0]).to.be.equal(ethUser.address); // receiver
            expect(royaltyInfo1[1]).to.be.equal(nftDetails.royalty.value); // value
            expect(tokenURI1).to.be.equal("");
            expect(duplicateNFTOwner2).to.be.equal(ethUser.address);
            expect(royaltyInfo2[0]).to.be.equal(ethUser.address); // receiver
            expect(royaltyInfo2[1]).to.be.equal(nftDetails.royalty.value); // value
            expect(tokenURI2).to.be.equal("");

            return [
                lockedEventDatas,
                duplicateCollectionAddresses,
                duplicateCollectionContracts,
            ];
        }

        async function lockOnEth(
            lockedEventDatas: TProcessedLogs[],
            duplicateCollectionContracts: ERC721Royalty[],
            duplicateCollectionAddresses: string[]
        ): Promise<
            [
                TProcessedLogs,
                TProcessedLogs,
                ContractTransactionReceipt | null,
                ContractTransactionReceipt | null
            ]
        > {
            const [duplicateCollectionAddress1, duplicateCollectionAddress2] =
                duplicateCollectionAddresses;

            await Promise.all(
                lockedEventDatas.map((data, i) =>
                    duplicateCollectionContracts[i]
                        .connect(ethUser)
                        .approve(ethBridge.address, data.tokenId)
                        .then((r) => r.wait())
                )
            );

            const [lockOnEthReceipt1, lockOnEthReceipt2] = await Promise.all(
                lockedEventDatas.map((data, i) =>
                    ethBridge.bridge
                        .connect(ethUser)
                        ["lock721"](
                            data.tokenId,
                            bscBridge.chainSymbol,
                            bscUser.address,
                            duplicateCollectionContracts[i]
                        )
                        .then((r) => r.wait())
                )
            );

            const originalStorageAddressForDuplicateCollectionProm1 =
                ethBridge.bridge["originalStorageMapping721"](
                    duplicateCollectionAddress1,
                    bscBridge.chainSymbol
                );

            const duplicateStorageAddressForDuplicateCollectionProm1 =
                ethBridge.bridge["duplicateStorageMapping721"](
                    duplicateCollectionAddress1,
                    ethBridge.chainSymbol
                );

            const originalStorageAddressForDuplicateCollectionProm2 =
                ethBridge.bridge["originalStorageMapping721"](
                    duplicateCollectionAddress2,
                    bscBridge.chainSymbol
                );

            const duplicateStorageAddressForDuplicateCollectionProm2 =
                ethBridge.bridge["duplicateStorageMapping721"](
                    duplicateCollectionAddress2,
                    ethBridge.chainSymbol
                );

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
            expect(lockedOnEthLogData1.tokenAmount).to.be.equal(1);
            expect(lockedOnEthLogData1.nftType).to.be.equal("singular");
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
            expect(lockedOnEthLogData2.tokenAmount).to.be.equal(1);
            expect(lockedOnEthLogData2.nftType).to.be.equal("singular");
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
            lockOnEthReceipt2: ContractTransactionReceipt | null
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

            let [owner1, owner2] = await Promise.all([
                mintedCollectionOnBSC.ownerOf(lockedOnEthLogData1.tokenId),
                mintedCollectionOnBSC.ownerOf(lockedOnEthLogData2.tokenId),
            ]);

            // ensure that storage is owner of the nft

            const [originalStorage721a, originalStorage721b] =
                await Promise.all([
                    bscBridge.bridge["originalStorageMapping721"](
                        mintedCollectionOnBSCAddress,
                        bscBridge.chainSymbol
                    ),
                    bscBridge.bridge["originalStorageMapping721"](
                        mintedCollectionOnBSCAddress,
                        bscBridge.chainSymbol
                    ),
                ]);
            expect(owner1).to.be.equal(originalStorage721a);
            expect(owner2).to.be.equal(originalStorage721b);

            await Promise.all(
                [claimDataArgs1, claimDataArgs2].map((args, i) =>
                    bscBridge.bridge
                        .connect(bscUser)
                        ["claimNFT721"](args, signatures[i], {
                            value: Fee.value,
                        })
                        .then((r) => r.wait())
                )
            );

            // ensure that bsc user is the owner after claiming
            [owner1, owner2] = await Promise.all([
                mintedCollectionOnBSC.ownerOf(lockedOnEthLogData1.tokenId),
                mintedCollectionOnBSC.ownerOf(lockedOnEthLogData2.tokenId),
            ]);
            expect(owner1).to.be.equal(bscUser.address);
            expect(owner2).to.be.equal(bscUser.address);
        }

        async function lockOnBSCAndClaimOnEth(): TLockOnBSCAndClaimOnEthReturn {
            const [parsedLogs1, parsedLogs2, lockedReceipt1, lockedReceipt2] =
                await lockOnBSC();

            return await claimOnEth(
                parsedLogs1,
                parsedLogs2,
                lockedReceipt1,
                lockedReceipt2
            );
        }

        async function lockOnEthAndClaimOnBSC(
            lockedEventDatas: TProcessedLogs[],
            duplicateCollectionContracts: ERC721Royalty[],
            duplicateCollectionAddresses: string[]
        ) {
            const [
                lockedOnEthLogData1,
                lockedOnEthLogData2,
                lockOnEthReceipt1,
                lockOnEthReceipt2,
            ] = await lockOnEth(
                lockedEventDatas,
                duplicateCollectionContracts,
                duplicateCollectionAddresses
            );

            await claimOnBSC(
                lockedOnEthLogData1,
                lockedOnEthLogData2,
                lockOnEthReceipt1,
                lockOnEthReceipt2
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
