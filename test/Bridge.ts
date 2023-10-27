import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { Bridge__factory } from "../contractsTypes";
import { TBridge, TGetValidatorSignatures } from "./types";
import {
    deploy1155Collection,
    deploy721Collection,
    lockOnBSCAndClaimOnEth,
    lockOnEthAndClaimOnBSC,
} from "./utils";

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

    const getValidatorSignatures: TGetValidatorSignatures = async (
        hash,
        type
    ) => {
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

    describe("Integration Tests; To and Fro between two chains", function () {
        it("Should successfully run the complete flow to and fro with multiple 1155 NFT", async function () {
            const cycles = 2;
            const nftType = 1155;

            const {
                mintedCollectionOnBSC,
                mintedCollectionOnBSCAddress,
                nftDetails,
                tokenIds,
            } = await deploy1155Collection(cycles, bscUser);

            for (let cycle = 0; cycle < cycles; cycle++) {
                const [
                    lockedEventDatas,
                    duplicateCollectionAddresses,
                    duplicateCollectionContracts,
                ] = await lockOnBSCAndClaimOnEth(
                    mintedCollectionOnBSC,
                    tokenIds,
                    mintedCollectionOnBSCAddress,
                    nftDetails,
                    bscUser,
                    ethUser,
                    bscBridge,
                    ethBridge,
                    nftType,
                    getValidatorSignatures
                );

                await lockOnEthAndClaimOnBSC(
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
                    nftType
                );
            }
        });

        it("Should successfully run the complete flow to and fro with multiple 721 NFT", async function () {
            const cycles = 2;
            const nftType = 721;

            const {
                mintedCollectionOnBSC,
                mintedCollectionOnBSCAddress,
                nftDetails,
                tokenIds,
            } = await deploy721Collection(bscUser);

            for (let cycle = 0; cycle < cycles; cycle++) {
                const [
                    lockedEventDatas,
                    duplicateCollectionAddresses,
                    duplicateCollectionContracts,
                ] = await lockOnBSCAndClaimOnEth(
                    mintedCollectionOnBSC,
                    tokenIds,
                    mintedCollectionOnBSCAddress,
                    nftDetails,
                    bscUser,
                    ethUser,
                    bscBridge,
                    ethBridge,
                    nftType,
                    getValidatorSignatures
                );

                await lockOnEthAndClaimOnBSC(
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
                    nftType
                );
            }
        });
    });
});
