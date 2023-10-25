import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import { EventLog, Typed, ZeroAddress, keccak256, solidityPacked } from "ethers";
import { ethers } from "hardhat";
import { Bridge, Bridge__factory, ERC721Royalty } from "../contractsTypes";

type TContractInstance = Bridge;
const encoder = new ethers.AbiCoder();

function hexStringToByteArray(hexString: string) {
    if (hexString.startsWith("0x")) {
        hexString = hexString.slice(2);
    }
    const byteArray = [];
    for (let i = 0; i < hexString.length; i += 2) {
        byteArray.push(parseInt(hexString.substr(i, 2), 16));
    }
    return new Uint8Array(byteArray);
}

describe("Bridge", function() {
    let Bridge: Bridge__factory,
        bscBridge: {
            bridge: TContractInstance;
            address: string,
            chainSymbol: string;
            collectionDeployer: string;
            storageDeployer: string;
        },
        ethBridge: {
            bridge: TContractInstance;
            chainSymbol: string;
            address: string,
            collectionDeployer: string;
            storageDeployer: string;
        };

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
        const address = await bridge.getAddress()
        return {
            address,
            bridge,
            chainSymbol,
            collectionDeployer,
            storageDeployer,
        };
    }

    beforeEach(async function() {
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

    describe("Deployment", function() {
        it("Should set the correct Collection Deployer address", async function() {
            expect(await bscBridge.bridge.collectionDeployer()).to.equal(
                bscBridge.collectionDeployer
            );
        });

        it("Should set the correct Storage Deployer address", async function() {
            expect(await bscBridge.bridge.storageDeployer()).to.equal(
                bscBridge.storageDeployer
            );
        });

        it("Should set validators correctly", async function() {
            for (let validator of bscValidators) {
                expect(await bscBridge.bridge.validators(validator)).to.equal(
                    true
                );
            }
        });

        it("Should set the correct Chain Symbol", async function() {
            expect(await bscBridge.bridge.selfChain()).to.be.equal("BSC");
        });

        it("Should fail if Collection Deployer address OR Storage Deployer address is address zero", async function() {
            expect(await bscBridge.bridge.collectionDeployer()).to.not.be.equal(
                ethers.ZeroAddress
            );
            expect(await bscBridge.bridge.storageDeployer()).to.not.be.equal(
                ethers.ZeroAddress
            );
        });

        it("Should fail to initialize contract if collection or storage address is zero", async function() {
            await expect(
                Bridge.deploy(
                    bscValidators,
                    bscBridge.chainSymbol,
                    ethers.ZeroAddress,
                    ethers.ZeroAddress
                )
            ).to.be.rejected;
        });

        it("Should have the correct validators count", async function() {
            expect(await bscBridge.bridge.validatorsCount()).to.be.equal(
                bscValidators.length
            );
        });
    });

    describe("Lock 721", function() {
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
        let tokenIds: Typed[] = []

        beforeEach(async function() {
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

            const toAddress = bscUser.address;
            const tokenId1 = ethers.Typed.uint256(1);
            const tokenId2 = ethers.Typed.uint256(2);
            tokenIds = [tokenId1, tokenId2]
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

            const mintPromises = tokenIds.map(id => {
                const mintArgs: Parameters<typeof mintedCollectionOnBSC.mint> = [
                    toAddress,
                    id,
                    royalty,
                    royaltyReceiver,
                    tokenURI,
                ];
                return mintedCollectionOnBSC
                    .connect(bscUser)
                    .mint(...mintArgs);
            });
            await Promise.all(mintPromises)
        });

        // it("Should deploy a new storage contract and transfer the nft to the storage contract", async function() {
        //     /// approve nft of token id minted in before each to be transferred to the bridge
        //     const approvedTx = await mintedCollectionOnBSC
        //         .connect(bscUser)
        //         .approve(
        //             await bscBridge.bridge.getAddress(),
        //             nftDetails.tokenId
        //         );
        //     await approvedTx.wait();
        //
        //     /// lock the nft by creating a new storage
        //     const tx = await bscBridge.bridge
        //         .connect(bscUser)
        //         .lock721(
        //             nftDetails.tokenId,
        //             "ETH",
        //             nftDetails.toAddress,
        //             nftDetails.collectionAddress
        //         );
        //     await tx.wait();
        //
        //     // get the new storage contract address for the original nft
        //     const storageAddressForCollection =
        //         await bscBridge.bridge.originalStorageMapping721(
        //             nftDetails.collectionAddress,
        //             "BSC"
        //         );
        //     expect(storageAddressForCollection).to.not.be.equal(
        //         ethers.ZeroAddress
        //     );
        //
        //     const owner = await mintedCollectionOnBSC.ownerOf(
        //         nftDetails.tokenId
        //     );
        //     expect(storageAddressForCollection).to.be.equal(owner);
        // });
        //
        it.only("Should claim nft on eth for corrosponding nft locked on bsc", async function() {
            const FEE = 5;
            /// approve nft of token id minted in before each to be transferred to the bridge
            tokenIds.map(id => {
                mintedCollectionOnBSC
                    .connect(bscUser)
                    .approve(
                        bscBridge.address,
                        id
                    )
                    .then((r) => r.wait());
            })

            /// lock the nft by creating a new storage
            const lockedReceipt = await bscBridge.bridge
                .connect(bscUser)
                .lock721(
                    nftDetails.tokenId1,
                    "ETH",
                    ethUser.address,
                    nftDetails.collectionAddress
                )
                .then((r) => r.wait());

            // get the new storage contract address for the original nft
            const storageAddressForCollection =
                await bscBridge.bridge.originalStorageMapping721(
                    nftDetails.collectionAddress,
                    "BSC"
                );
            expect(storageAddressForCollection).to.not.be.equal(
                ethers.ZeroAddress
            );

            const owner = await mintedCollectionOnBSC.ownerOf(
                nftDetails.tokenId1
            );
            expect(storageAddressForCollection).to.be.equal(owner);

            // =================================================================

            // get logs emitted after locking
            /* 
                emit Locked(
                tokenId,
                destinationChain,
                destinationUserAddress,
                address(sourceNftContractAddress),
                1,
                TYPEERC721,
                selfChain
                );
            */

            const logs = lockedReceipt?.logs[1] as EventLog;
            const args = logs.args;

            const lockedEventData = {
                tokenId: Number(args[0]),
                destinationChain: args[1],
                destinationUserAddress: args[2],
                sourceNftContractAddress: args[3],
                tokenAmount: Number(args[4]),
                nftType: args[5],
                sourceChain: args[6],
            };

            // ======================== PREPARE DATA FOR HASHING ======================
            // create hash from log data and nft details

            /**
             * return
                keccak256(
                    abi.encode(
                        data.tokenId,
                        data.sourceChain,
                        data.destinationChain,
                        data.destinationUserAddress,
                        data.sourceNftContractAddress,
                        data.name,
                        data.symbol,
                        data.royalty,
                        data.royaltyReceiver,
                        data.metadata,
                        data.transactionHash,
                        data.tokenAmount,
                        data.nftType,
                        data.fee
                    )
                );
             */
            const lockHashBSC = lockedReceipt?.hash;
            const fee = ethers.Typed.uint256(FEE);

            const claimDataArgs: Parameters<
                typeof ethBridge.bridge.claimNFT721
            >[0] = {
                tokenId: lockedEventData.tokenId,
                sourceChain: lockedEventData.sourceChain,
                destinationChain: lockedEventData.destinationChain,
                destinationUserAddress: lockedEventData.destinationUserAddress,
                sourceNftContractAddress:
                    lockedEventData.sourceNftContractAddress,
                name: nftDetails.name,
                symbol: nftDetails.symbol,
                royalty: nftDetails.royalty.value,
                royaltyReceiver: ethUser.address,
                metadata: nftDetails.tokenURI,
                transactionHash: lockHashBSC ?? "",
                tokenAmount: lockedEventData.tokenAmount,
                nftType: lockedEventData.nftType,
                fee: fee.value,
            };
            const nftTransferDetailsTypes = [
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

            const nftTransferDetailsValues = Object.values(claimDataArgs);

            // ======================== HASH AND SEND ======================

            const dataHash = keccak256(
                encoder.encode(
                    nftTransferDetailsTypes,
                    nftTransferDetailsValues
                )
            );

            const _1_validatorSignatureProm = ethValidator1.signMessage(
                hexStringToByteArray(dataHash)
            );
            const _2_validatorSignatureProm = ethValidator2.signMessage(
                hexStringToByteArray(dataHash)
            );

            const [_1_validatorSignature, _2_validatorSignature] =
                await Promise.all([
                    _1_validatorSignatureProm,
                    _2_validatorSignatureProm,
                ]);

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT721(
                    claimDataArgs,
                    [_1_validatorSignature, _2_validatorSignature],
                    { value: fee.value }
                )
                .then((r) => r.wait());

            // ======================= VERIFY ===================

            const [destinationChainId, duplicateCollectionAddress] =
                await ethBridge.bridge.originalToDuplicateMapping(
                    lockedEventData.sourceNftContractAddress,
                    lockedEventData.sourceChain
                );

            expect(duplicateCollectionAddress).to.not.be.equal(ZeroAddress);
            expect(destinationChainId).to.be.equal(ethBridge.chainSymbol);

            const duplicateCollectionContract = await ethers.getContractAt(
                "ERC721Royalty",
                duplicateCollectionAddress
            );
            const duplicateNFTOwnerProm = duplicateCollectionContract.ownerOf(
                ethers.Typed.uint256(lockedEventData.tokenId)
            );

            const royaltyInfoProm = duplicateCollectionContract.royaltyInfo(
                ethers.Typed.uint256(lockedEventData.tokenId),
                ethers.Typed.uint256(1)
            );

            const tokenURIProm = duplicateCollectionContract.tokenURI(
                ethers.Typed.uint256(lockedEventData.tokenId)
            );

            const [duplicateNFTOwner, royaltyInfo, tokenURI] =
                await Promise.all([
                    duplicateNFTOwnerProm,
                    royaltyInfoProm,
                    tokenURIProm,
                ]);

            expect(duplicateNFTOwner).to.be.equal(ethUser.address);
            expect(royaltyInfo[0]).to.be.equal(ethUser.address); // receiver
            expect(royaltyInfo[1]).to.be.equal(nftDetails.royalty.value); // value
            expect(tokenURI).to.be.equal("");

            // ======================= LOCK ON ETH ===================
            await duplicateCollectionContract
                .connect(ethUser)
                .approve(
                    await ethBridge.bridge.getAddress(),
                    lockedEventData.tokenId
                )
                .then((r) => r.wait());

            const lockOnEthReceipt = await ethBridge.bridge
                .connect(ethUser)
                .lock721(
                    lockedEventData.tokenId,
                    bscBridge.chainSymbol,
                    bscUser.address,
                    duplicateCollectionAddress
                )
                .then((r) => r.wait());

            const originalStorageAddressForDuplicateCollectionProm =
                ethBridge.bridge.originalStorageMapping721(
                    duplicateCollectionAddress,
                    bscBridge.chainSymbol
                );

            const duplicateStorageAddressForDuplicateCollectionProm =
                ethBridge.bridge.duplicateStorageMapping721(
                    duplicateCollectionAddress,
                    ethBridge.chainSymbol
                );

            const [
                originalStorageAddressForDuplicateCollection,
                duplicateStorageAddressForDuplicateCollection,
            ] = await Promise.all([
                originalStorageAddressForDuplicateCollectionProm,
                duplicateStorageAddressForDuplicateCollectionProm,
            ]);

            // ======================= LOCK ON ETH - VERIFY ===================

            expect(originalStorageAddressForDuplicateCollection).to.be.equal(
                ZeroAddress
            );
            expect(
                duplicateStorageAddressForDuplicateCollection
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
            // @ts-ignore
            const lockedOnEthLogs: EventLog = lockOnEthReceipt.logs[1];
            const lockedOnEthLogsArgs = lockedOnEthLogs.args;

            const lockedOnEthLogData = {
                tokenId: Number(lockedOnEthLogsArgs[0]),
                destinationChain: lockedOnEthLogsArgs[1],
                destinationUserAddress: lockedOnEthLogsArgs[2],
                sourceNftContractAddress: lockedOnEthLogsArgs[3],
                tokenAmount: Number(lockedOnEthLogsArgs[4]),
                nftType: lockedOnEthLogsArgs[5],
                sourceChain: lockedOnEthLogsArgs[6],
            };

            expect(lockedOnEthLogData.tokenId).to.be.equal(
                nftDetails.tokenId1.value
            );
            expect(lockedOnEthLogData.destinationChain).to.be.equal(
                bscBridge.chainSymbol
            );
            expect(lockedOnEthLogData.destinationUserAddress).to.be.equal(
                bscUser.address
            );
            expect(lockedOnEthLogData.sourceNftContractAddress).to.be.equal(
                nftDetails.collectionAddress
            );
            expect(lockedOnEthLogData.tokenAmount).to.be.equal(1);
            expect(lockedOnEthLogData.nftType).to.be.equal("singular");
            expect(lockedOnEthLogData.sourceChain).to.be.equal(
                bscBridge.chainSymbol
            );

            // ======================= CLAIM ON BSC ===================
            {
                const lockHashETH = lockOnEthReceipt?.hash ?? "";
                const feeOnBSC = ethers.Typed.uint256(FEE);

                const claimDataArgs: Parameters<
                    typeof ethBridge.bridge.claimNFT721
                >[0] = {
                    tokenId: lockedOnEthLogData.tokenId,
                    sourceChain: lockedOnEthLogData.sourceChain,
                    destinationChain: lockedOnEthLogData.destinationChain,
                    destinationUserAddress:
                        lockedOnEthLogData.destinationUserAddress,
                    sourceNftContractAddress:
                        lockedOnEthLogData.sourceNftContractAddress,
                    name: nftDetails.name,
                    symbol: nftDetails.symbol,
                    royalty: nftDetails.royalty.value,
                    royaltyReceiver: ethUser.address,
                    metadata: nftDetails.tokenURI,
                    transactionHash: lockHashETH,
                    tokenAmount: lockedOnEthLogData.tokenAmount,
                    nftType: lockedOnEthLogData.nftType,
                    fee: feeOnBSC.value,
                };

                const nftTransferDetailsValues = Object.values(claimDataArgs);

                const dataHash = keccak256(
                    encoder.encode(
                        nftTransferDetailsTypes,
                        nftTransferDetailsValues
                    )
                );

                const _1_validatorSignatureProm = bscValidator1.signMessage(
                    hexStringToByteArray(dataHash)
                );
                const _2_validatorSignatureProm = bscValidator2.signMessage(
                    hexStringToByteArray(dataHash)
                );

                const [_1_validatorSignature, _2_validatorSignature] =
                    await Promise.all([
                        _1_validatorSignatureProm,
                        _2_validatorSignatureProm,
                    ]);

                let owner = await mintedCollectionOnBSC.ownerOf(
                    lockedOnEthLogData.tokenId
                );

                // ensure that storage is owner of the nft
                const originalStorage721 =
                    await bscBridge.bridge.originalStorageMapping721(
                        await mintedCollectionOnBSC.getAddress(),
                        bscBridge.chainSymbol
                    );
                expect(owner).to.be.equal(originalStorage721);

                await bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT721(
                        claimDataArgs,
                        [_1_validatorSignature, _2_validatorSignature],
                        { value: fee.value }
                    )
                    .then((r) => r.wait());

                // ensure that bsc user is the owner after claiming
                owner = await mintedCollectionOnBSC.ownerOf(
                    lockedOnEthLogData.tokenId
                );
                expect(owner).to.be.equal(bscUser.address);
            }

            {
                /// approve nft of token id minted in before each to be transferred to the bridge
                await mintedCollectionOnBSC
                    .connect(bscUser)
                    .approve(
                        await bscBridge.bridge.getAddress(),
                        nftDetails.tokenId1
                    )
                    .then((r) => r.wait());

                /// lock the nft by creating a new storage
                const lockedReceipt = await bscBridge.bridge
                    .connect(bscUser)
                    .lock721(
                        nftDetails.tokenId1,
                        ethBridge.chainSymbol,
                        ethUser.address,
                        nftDetails.collectionAddress
                    )
                    .then((r) => r.wait());

                // get the new storage contract address for the original nft
                const storageAddressForCollection =
                    await bscBridge.bridge.originalStorageMapping721(
                        nftDetails.collectionAddress,
                        "BSC"
                    );
                expect(storageAddressForCollection).to.not.be.equal(
                    ethers.ZeroAddress
                );

                const owner = await mintedCollectionOnBSC.ownerOf(
                    nftDetails.tokenId1
                );
                expect(storageAddressForCollection).to.be.equal(owner);
                // =================================================================

                // get logs emitted after locking
                /* 
                emit Locked(
                tokenId,
                destinationChain,
                destinationUserAddress,
                address(sourceNftContractAddress),
                1,
                TYPEERC721,
                selfChain
                );
            */

                const logs = lockedReceipt?.logs[1] as EventLog;
                const args = logs.args;

                const lockedEventData = {
                    tokenId: Number(args[0]),
                    destinationChain: args[1],
                    destinationUserAddress: args[2],
                    sourceNftContractAddress: args[3],
                    tokenAmount: Number(args[4]),
                    nftType: args[5],
                    sourceChain: args[6],
                };

                expect(lockedEventData.tokenId).to.be.equal(
                    nftDetails.tokenId1.value
                );
                expect(lockedEventData.destinationChain).to.be.equal(
                    ethBridge.chainSymbol
                );
                expect(lockedEventData.destinationUserAddress).to.be.equal(
                    ethUser.address
                );
                expect(lockedEventData.sourceNftContractAddress).to.be.equal(
                    await mintedCollectionOnBSC.getAddress()
                );
                expect(lockedEventData.tokenAmount).to.be.equal(1);
                expect(lockedEventData.nftType).to.be.equal("singular");
                expect(lockedEventData.sourceChain).to.be.equal(
                    bscBridge.chainSymbol
                );
                // ======================== PREPARE DATA FOR HASHING ======================
                // create hash from log data and nft details

                /**
             * return
                keccak256(
                    abi.encode(
                        data.tokenId,
                        data.sourceChain,
                        data.destinationChain,
                        data.destinationUserAddress,
                        data.sourceNftContractAddress,
                        data.name,
                        data.symbol,
                        data.royalty,
                        data.royaltyReceiver,
                        data.metadata,
                        data.transactionHash,
                        data.tokenAmount,
                        data.nftType,
                        data.fee
                    )
                );
             */
                const lockHashBSC = lockedReceipt?.hash;
                const fee = ethers.Typed.uint256(FEE);

                const claimDataArgs: Parameters<
                    typeof ethBridge.bridge.claimNFT721
                >[0] = {
                    tokenId: lockedEventData.tokenId,
                    sourceChain: lockedEventData.sourceChain,
                    destinationChain: lockedEventData.destinationChain,
                    destinationUserAddress:
                        lockedEventData.destinationUserAddress,
                    sourceNftContractAddress:
                        lockedEventData.sourceNftContractAddress,
                    name: nftDetails.name,
                    symbol: nftDetails.symbol,
                    royalty: nftDetails.royalty.value,
                    royaltyReceiver: ethUser.address,
                    metadata: nftDetails.tokenURI,
                    transactionHash: lockHashBSC ?? "",
                    tokenAmount: lockedEventData.tokenAmount,
                    nftType: lockedEventData.nftType,
                    fee: fee.value,
                };
                // ======================== HASH AND SEND ======================
                const nftTransferDetailsValues = Object.values(claimDataArgs);

                const dataHash = keccak256(
                    encoder.encode(
                        nftTransferDetailsTypes,
                        nftTransferDetailsValues
                    )
                );

                const _1_validatorSignatureProm = ethValidator1.signMessage(
                    hexStringToByteArray(dataHash)
                );
                const _2_validatorSignatureProm = ethValidator2.signMessage(
                    hexStringToByteArray(dataHash)
                );

                const [_1_validatorSignature, _2_validatorSignature] =
                    await Promise.all([
                        _1_validatorSignatureProm,
                        _2_validatorSignatureProm,
                    ]);

                await ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(
                        claimDataArgs,
                        [_1_validatorSignature, _2_validatorSignature],
                        { value: fee.value }
                    )
                    .then((r) => r.wait());

                // ensure that the user gets the unlocked nft
                const ownerOfEthDuplicate = await duplicateCollectionContract.ownerOf(nftDetails.tokenId1.value)
                expect(ownerOfEthDuplicate).to.be.equal(ethUser.address)
            }

            // lock duplicate on eth and claim on bsc
            {
                await duplicateCollectionContract
                    .connect(ethUser)
                    .approve(
                        await ethBridge.bridge.getAddress(),
                        lockedEventData.tokenId
                    )
                    .then((r) => r.wait());

                const lockOnEthReceipt = await ethBridge.bridge
                    .connect(ethUser)
                    .lock721(
                        lockedEventData.tokenId,
                        bscBridge.chainSymbol,
                        bscUser.address, // receiver on bsc side
                        duplicateCollectionAddress
                    )
                    .then((r) => r.wait());

                const originalStorageAddressForDuplicateCollectionProm =
                    ethBridge.bridge.originalStorageMapping721(
                        duplicateCollectionAddress,
                        bscBridge.chainSymbol
                    );

                const duplicateStorageAddressForDuplicateCollectionProm =
                    ethBridge.bridge.duplicateStorageMapping721(
                        duplicateCollectionAddress,
                        ethBridge.chainSymbol
                    );

                const [
                    originalStorageAddressForDuplicateCollection,
                    duplicateStorageAddressForDuplicateCollection,
                ] = await Promise.all([
                    originalStorageAddressForDuplicateCollectionProm,
                    duplicateStorageAddressForDuplicateCollectionProm,
                ]);

                // ======================= LOCK ON ETH - VERIFY ===================

                expect(originalStorageAddressForDuplicateCollection).to.be.equal(
                    ZeroAddress
                );
                expect(
                    duplicateStorageAddressForDuplicateCollection
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
                // @ts-ignore
                const lockedOnEthLogs: EventLog = lockOnEthReceipt.logs[1];
                const lockedOnEthLogsArgs = lockedOnEthLogs.args;

                const lockedOnEthLogData = {
                    tokenId: Number(lockedOnEthLogsArgs[0]),
                    destinationChain: lockedOnEthLogsArgs[1],
                    destinationUserAddress: lockedOnEthLogsArgs[2],
                    sourceNftContractAddress: lockedOnEthLogsArgs[3],
                    tokenAmount: Number(lockedOnEthLogsArgs[4]),
                    nftType: lockedOnEthLogsArgs[5],
                    sourceChain: lockedOnEthLogsArgs[6],
                };

                expect(lockedOnEthLogData.tokenId).to.be.equal(
                    nftDetails.tokenId1.value
                );
                expect(lockedOnEthLogData.destinationChain).to.be.equal(
                    bscBridge.chainSymbol
                );
                expect(lockedOnEthLogData.destinationUserAddress).to.be.equal(
                    bscUser.address
                );
                expect(lockedOnEthLogData.sourceNftContractAddress).to.be.equal(
                    nftDetails.collectionAddress
                );
                expect(lockedOnEthLogData.tokenAmount).to.be.equal(1);
                expect(lockedOnEthLogData.nftType).to.be.equal("singular");
                expect(lockedOnEthLogData.sourceChain).to.be.equal(
                    bscBridge.chainSymbol
                );

                // ======================= CLAIM ON BSC ===================
                const lockHashETH = lockOnEthReceipt?.hash ?? "";
                const feeOnBSC = ethers.Typed.uint256(FEE);

                const claimDataArgs: Parameters<
                    typeof ethBridge.bridge.claimNFT721
                >[0] = {
                    tokenId: lockedOnEthLogData.tokenId,
                    sourceChain: lockedOnEthLogData.sourceChain,
                    destinationChain: lockedOnEthLogData.destinationChain,
                    destinationUserAddress:
                        lockedOnEthLogData.destinationUserAddress,
                    sourceNftContractAddress:
                        lockedOnEthLogData.sourceNftContractAddress,
                    name: nftDetails.name,
                    symbol: nftDetails.symbol,
                    royalty: nftDetails.royalty.value,
                    royaltyReceiver: ethUser.address,
                    metadata: nftDetails.tokenURI,
                    transactionHash: lockHashETH,
                    tokenAmount: lockedOnEthLogData.tokenAmount,
                    nftType: lockedOnEthLogData.nftType,
                    fee: feeOnBSC.value,
                };

                const nftTransferDetailsValues = Object.values(claimDataArgs);

                const dataHash = keccak256(
                    encoder.encode(
                        nftTransferDetailsTypes,
                        nftTransferDetailsValues
                    )
                );

                const _1_validatorSignatureProm = bscValidator1.signMessage(
                    hexStringToByteArray(dataHash)
                );
                const _2_validatorSignatureProm = bscValidator2.signMessage(
                    hexStringToByteArray(dataHash)
                );

                const [_1_validatorSignature, _2_validatorSignature] =
                    await Promise.all([
                        _1_validatorSignatureProm,
                        _2_validatorSignatureProm,
                    ]);

                let owner = await mintedCollectionOnBSC.ownerOf(
                    lockedOnEthLogData.tokenId
                );

                // ensure that storage is owner of the nft
                const originalStorage721 =
                    await bscBridge.bridge.originalStorageMapping721(
                        await mintedCollectionOnBSC.getAddress(),
                        bscBridge.chainSymbol
                    );
                expect(owner).to.be.equal(originalStorage721);

                await bscBridge.bridge
                    .connect(bscUser)
                    .claimNFT721(
                        claimDataArgs,
                        [_1_validatorSignature, _2_validatorSignature],
                        { value: fee.value }
                    )
                    .then((r) => r.wait());

                // ensure that bsc user is the owner after claiming
                owner = await mintedCollectionOnBSC.ownerOf(
                    lockedOnEthLogData.tokenId
                );
                expect(owner).to.be.equal(bscUser.address);
            }
        });

        it.only("", async function() { });
    });
});
