import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import { EventLog, ZeroAddress, keccak256 } from "ethers";
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

describe("Bridge", function () {
    let Bridge: Bridge__factory,
        bscBridge: {
            bridge: TContractInstance;
            chainSymbol: string;
            collectionDeployer: string;
            storageDeployer: string;
        },
        ethBridge: {
            bridge: TContractInstance;
            chainSymbol: string;
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

    let validators: string[];

    let wallet1: any, wallet2: any;

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

        return {
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
            for (let validator of validators) {
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
                    validators,
                    bscBridge.chainSymbol,
                    ethers.ZeroAddress,
                    ethers.ZeroAddress
                )
            ).to.be.rejected;
        });

        it("Should have the correct validators count", async function () {
            expect(await bscBridge.bridge.validatorsCount()).to.be.equal(
                validators.length
            );
        });
    });

    describe.only("Lock 721", function () {
        const uint256 = ethers.Typed.uint256;

        let nftDetails = {
            toAddress: "",
            tokenId: uint256(0),
            royalty: uint256(0),
            royaltyReceiver: "",
            tokenURI: "",
            collectionAddress: "",
            name: "",
            symbol: "",
        };

        let mintedCollectionOnBSC: ERC721Royalty;

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

            const toAddress = bscUser.address;
            const tokenId = ethers.Typed.uint256(1);
            const royalty = ethers.Typed.uint256(100);
            const royaltyReceiver = bscUser.address;
            const tokenURI = "";
            const mintArgs: Parameters<typeof mintedCollectionOnBSC.mint> = [
                toAddress,
                tokenId,
                royalty,
                royaltyReceiver,
                tokenURI,
            ];

            nftDetails = {
                toAddress,
                tokenId,
                royalty,
                royaltyReceiver,
                tokenURI,
                collectionAddress: newCollectionAddress,
                name,
                symbol,
            };

            const mintedNFT = await mintedCollectionOnBSC
                .connect(bscUser)
                .mint(...mintArgs);
            await mintedNFT.wait();
        });

        it("Should deploy a new storage contract and transfer the nft to the storage contract", async function () {
            /// approve nft of token id minted in before each to be transferred to the bridge
            const approvedTx = await mintedCollectionOnBSC
                .connect(bscUser)
                .approve(
                    await bscBridge.bridge.getAddress(),
                    nftDetails.tokenId
                );
            await approvedTx.wait();

            /// lock the nft by creating a new storage
            const tx = await bscBridge.bridge
                .connect(bscUser)
                .lock721(
                    nftDetails.tokenId,
                    "ETH",
                    nftDetails.toAddress,
                    nftDetails.collectionAddress
                );
            await tx.wait();

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
                nftDetails.tokenId
            );
            expect(storageAddressForCollection).to.be.equal(owner);
        });

        it("Should claim nft on eth for corrosponding nft locked on bsc", async function () {
            /// approve nft of token id minted in before each to be transferred to the bridge
            const approvedTx = await mintedCollectionOnBSC
                .connect(bscUser)
                .approve(
                    await bscBridge.bridge.getAddress(),
                    nftDetails.tokenId
                );
            await approvedTx.wait();

            /// lock the nft by creating a new storage
            const lockedTx = await bscBridge.bridge
                .connect(bscUser)
                .lock721(
                    nftDetails.tokenId,
                    "ETH",
                    nftDetails.toAddress,
                    nftDetails.collectionAddress
                );
            const lockedReceipt = await lockedTx.wait();

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
                nftDetails.tokenId
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
            console.log({ args });

            const lockedEventData = {
                tokenId: Number(args[0]),
                destinationChain: args[1],
                destinationUserAddress: args[2],
                sourceNftContractAddress: args[3],
                tokenAmount: Number(args[4]),
                nftType: args[5],
                sourceChain: args[6],
            };

            // =================================================================
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
            const lockHash = lockedReceipt?.hash;
            const fee = ethers.Typed.uint256(5);

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
                royalty: 100,
                royaltyReceiver: nftDetails.royaltyReceiver,
                metadata: nftDetails.tokenURI,
                transactionHash: lockHash ?? "",
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

            const dataHash = keccak256(
                encoder.encode(
                    nftTransferDetailsTypes,
                    nftTransferDetailsValues
                )
            );

            const _1_validatorSignature = await ethValidator1.signMessage(
                hexStringToByteArray(dataHash)
            );
            const _2_validatorSignature = await ethValidator2.signMessage(
                hexStringToByteArray(dataHash)
            );

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT721(
                    claimDataArgs,
                    [_1_validatorSignature, _2_validatorSignature],
                    { value: fee.value }
                )
                .then((r) => r.wait());

            const duplicateCollection =
                await ethBridge.bridge.originalToDuplicateMapping(
                    lockedEventData.sourceNftContractAddress,
                    lockedEventData.sourceChain
                );

            expect(duplicateCollection).to.not.be.equal(ZeroAddress);

            console.log({ duplicateCollection });
        });
    });
});
