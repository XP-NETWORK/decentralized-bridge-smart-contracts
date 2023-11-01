import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import { ethers } from "hardhat";
import {
    Bridge__factory,
    ERC1155Royalty,
    ERC721Royalty,
} from "../contractsTypes";
import {
    TBridge,
    TChainArr,
    TGetValidatorSignatures,
    TProcessedLogs,
} from "./types";
import {
    claimOnBSC,
    claim,
    deploy1155Collection,
    deploy721Collection,
    encoder,
    hexStringToByteArray,
    lockOnBSCAndClaimOnEth,
    lockOnEth,
    lockOnEthAndClaimOnBSC,
    lock,
} from "./utils";
import {
    Contract,
    ContractTransactionReceipt,
    EventLog,
    Typed,
    ZeroAddress,
    keccak256,
} from "ethers";

describe("Bridge", function () {
    let Bridge: Bridge__factory, bscBridge: TBridge, ethBridge: TBridge;

    let bscValidator1: HardhatEthersSigner,
        bscValidator2: HardhatEthersSigner,
        ethValidator1: HardhatEthersSigner,
        ethValidator2: HardhatEthersSigner,
        bscUser: HardhatEthersSigner,
        ethUser: HardhatEthersSigner,
        ethBridgeDeployer: HardhatEthersSigner,
        bscBridgeDeployer: HardhatEthersSigner,
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
            bscBridgeDeployer,
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
            bscBridgeDeployer
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
                expect(
                    (await bscBridge.bridge.validators(validator)).added
                ).to.equal(true);
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

    describe("claimValidatorRewards", async function () {
        const createClaimValidatorHash = async (validatorAddress: string) => {
            const hash = keccak256(
                encoder.encode(["address"], [validatorAddress])
            );
            const hexifiedDataHash = hexStringToByteArray(hash);
            const signatures = await getValidatorSignatures(
                hexifiedDataHash,
                "eth"
            );
            return signatures;
        };

        it("Should not be able to claim rewards if validator address is zero address", async function () {
            const signatures = await createClaimValidatorHash(ZeroAddress);

            await expect(
                ethBridge.bridge
                    .claimValidatorRewards(ZeroAddress, signatures)
                    .then((r) => r.wait())
            ).to.be.revertedWith("Address cannot be zero address!");
        });

        it("Should fail if no signatures are provided", async function () {
            const newValidator = addrs[10];

            await expect(
                bscBridge.bridge
                    .claimValidatorRewards(newValidator, [])
                    .then((r) => r.wait())
            ).to.be.revertedWith("Must have signatures!");
        });

        it("Should fail if validators do not reach threshold", async function () {
            const signatures = await createClaimValidatorHash(
                ethValidator1.address
            );

            await expect(
                ethBridge.bridge
                    .claimValidatorRewards(ethValidator1.address, [
                        signatures[0],
                    ])
                    .then((r) => r.wait())
            ).to.be.revertedWith("Threshold not reached!");
        });

        it("should successfully transfer funds when validator claims rewards", async function () {
            let data = {
                tokenId: 2,
                sourceChain: "BSC",
                destinationChain: "ETH",
                destinationUserAddress:
                    "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
                sourceNftContractAddress:
                    "0xE3C0bf45000b24D537D9760adf81718fF197D630",
                name: "MyCollection",
                symbol: "MC",
                royalty: 100,
                royaltyReceiver: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
                metadata: "",
                transactionHash:
                    "0x6de34e976f3107b37ed992c8aaa6e5f04b3b5a2f127a3256f5b187ad45103911",
                tokenAmount: 1,
                nftType: "multiple",
                fee: 5,
            };
            const signatures = [
                "0x0d43ae15e75d59492c4952bd7465525d16e9970c237d3847d6867fdb4a4a537a525a38ce04e056540ce64fa7ae8aaa7638cd07a5c0fc04ecd78be4e7ac3f99501c",
                "0x204e0cf223bc114ea2785bdf56e1d9e2b10a299e8e5294cb2d525fd1bbb04fbc369a9edac7684c2e5b5df6715d07b69077593391c3483d580bd4dcd979485a811b",
            ];

            let [validator1, bridgeBalance, validator_1_balance] =
                await Promise.all([
                    ethBridge.bridge.validators(ethValidator1.address),
                    ethers.provider.getBalance(ethBridge.address),
                    ethers.provider.getBalance(ethValidator1.address),
                ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("0"));
            expect(bridgeBalance).to.be.eq(BigInt("0"));
            expect(validator_1_balance).to.be.eq(
                BigInt("10000000000000000000000")
            );

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT1155(data, signatures, {
                    value: 8,
                });

            [validator1, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("4"));
            expect(bridgeBalance).to.be.eq(BigInt("8"));

            const claimSignatures = await createClaimValidatorHash(
                ethValidator1.address
            );
            await ethBridge.bridge
                .claimValidatorRewards(ethValidator1.address, claimSignatures)
                .then((r) => r.wait());

            validator_1_balance = await ethers.provider.getBalance(
                ethValidator1.address
            );

            expect(validator_1_balance).to.be.eq(
                BigInt("10000000000000000000004")
            );
        });
    });

    describe("addValidator", async function () {
        const createAddValidatorHash = async (validatorAddress: string) => {
            const hash = keccak256(
                encoder.encode(["address"], [validatorAddress])
            );
            const hexifiedDataHash = hexStringToByteArray(hash);
            const signatures = await getValidatorSignatures(
                hexifiedDataHash,
                "bsc"
            );
            return signatures;
        };
        it("Should fail if zero address for validator is provided", async function () {
            const signatures = await createAddValidatorHash(ZeroAddress);

            await expect(
                bscBridge.bridge
                    .addValidator(ZeroAddress, signatures)
                    .then((r) => r.wait())
            ).to.be.revertedWith("Address cannot be zero address!");
        });

        it("Should fail if no signatures are provided", async function () {
            const newValidator = addrs[10];

            await expect(
                bscBridge.bridge
                    .addValidator(newValidator, [])
                    .then((r) => r.wait())
            ).to.be.revertedWith("Must have signatures!");
        });

        it("Should fail if validators do not reach threshold", async function () {
            const newValidator = addrs[10];
            const signatures = await createAddValidatorHash(
                newValidator.address
            );

            await expect(
                bscBridge.bridge
                    .addValidator(newValidator.address, [signatures[0]])
                    .then((r) => r.wait())
            ).to.be.revertedWith("Threshold not reached!");
        });

        it("Should successfully add a new validator with correct arguments", async function () {
            const newValidator = addrs[10];

            const [signatures, beforeValidatorAdditionCount] =
                await Promise.all([
                    createAddValidatorHash(newValidator.address),
                    bscBridge.bridge.validatorsCount(),
                ]);

            const receipt = await bscBridge.bridge
                .addValidator(newValidator.address, signatures)
                .then((r) => r.wait());

            const logs = receipt?.logs?.[0] as EventLog;

            expect(logs).to.not.be.undefined;
            expect(logs.args).to.not.be.undefined;

            const logsArgs = logs.args[0];

            expect(logsArgs).to.be.eq(newValidator.address);

            const [validatorExistsInMapping, afterValidatorAdditionCount] =
                await Promise.all([
                    bscBridge.bridge.validators(newValidator),
                    bscBridge.bridge.validatorsCount(),
                ]);

            expect(validatorExistsInMapping.added).to.be.eq(true);
            expect(Number(afterValidatorAdditionCount)).to.be.eq(
                Number(beforeValidatorAdditionCount) + 1
            );
        });
    });

    describe("lock721", async function () {
        const DestinationChain = "";
        const DestinationUserAddress = "";
        let User: HardhatEthersSigner;
        let mintedCollectionOnBSC: ERC721Royalty;
        let mintedCollectionOnBSCAddress: string;
        let tokenIds: [Typed, Typed];

        this.beforeEach(async function () {
            User = addrs[10];

            const res = await deploy721Collection(bscUser);
            mintedCollectionOnBSC = res.mintedCollectionOnBSC;
            mintedCollectionOnBSCAddress = res.mintedCollectionOnBSCAddress;
            tokenIds = res.tokenIds;

            await mintedCollectionOnBSC
                .connect(bscUser)
                .setApprovalForAll(bscBridge.address, true)
                .then((r) => r.wait());
        });

        it("should fail to lock 721 NFT if sourceNftContractAddress is zero address", async function () {
            const tokenId = 1;
            await expect(
                bscBridge.bridge
                    .connect(User)
                    .lock721(
                        tokenId,
                        DestinationChain,
                        DestinationUserAddress,
                        ZeroAddress
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWith(
                "sourceNftContractAddress cannot be zero address"
            );
        });

        it("should fail to lock 721 NFT if caller is not NFT owner", async function () {
            await expect(
                bscBridge.bridge
                    .connect(User)
                    .lock721(
                        tokenIds[0],
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWithCustomError(
                mintedCollectionOnBSC,
                "ERC721IncorrectOwner"
            );
        });

        it("should fail to lock 721 NFT if token id does not exist", async function () {
            const TOKEN_ID_ATTEMPT_TO_LOCK = 5;

            await expect(
                bscBridge.bridge
                    .connect(bscUser)
                    .lock721(
                        TOKEN_ID_ATTEMPT_TO_LOCK,
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWithCustomError(
                mintedCollectionOnBSC,
                `ERC721NonexistentToken`
            );
        });

        it("should fail to lock 721 NFT if token type does not match", async function () {
            const TOKEN_ID_ATTEMPT_TO_LOCK = 5;
            const { mintedCollectionOnBSCAddress, mintedCollectionOnBSC } =
                await deploy1155Collection(2, bscUser);

            await mintedCollectionOnBSC
                .connect(bscUser)
                .setApprovalForAll(bscBridge.address, true)
                .then((r) => r.wait());

            await expect(
                bscBridge.bridge
                    .connect(bscUser)
                    .lock721(
                        TOKEN_ID_ATTEMPT_TO_LOCK,
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress
                    )
                    .then((r) => r.wait())
            ).to.be.reverted;
        });
    });

    describe("lock1155", async function () {
        const DestinationChain = "";
        const DestinationUserAddress = "";
        let User: HardhatEthersSigner;
        let mintedCollectionOnBSC: ERC1155Royalty;
        let mintedCollectionOnBSCAddress: string;

        this.beforeEach(async function () {
            User = addrs[10];

            const res = await deploy1155Collection(2, bscUser);
            mintedCollectionOnBSCAddress = res.mintedCollectionOnBSCAddress;
            mintedCollectionOnBSC = res.mintedCollectionOnBSC;

            await mintedCollectionOnBSC
                .connect(bscUser)
                .setApprovalForAll(bscBridge.address, true)
                .then((r) => r.wait());
        });

        it("should fail to lock 1155 NFT if sourceNftContractAddress is zero address", async function () {
            const tokenId = 1;
            const tokenAmount = 1;

            await expect(
                bscBridge.bridge
                    .connect(User)
                    .lock1155(
                        tokenId,
                        DestinationChain,
                        DestinationUserAddress,
                        ZeroAddress,
                        tokenAmount
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWith(
                "sourceNftContractAddress cannot be zero address"
            );
        });

        it("should fail to lock 1155 NFT if amount is zero", async function () {
            const tokenId = 1;
            const tokenAmount = 0;
            await expect(
                bscBridge.bridge
                    .connect(User)
                    .lock1155(
                        tokenId,
                        DestinationChain,
                        DestinationUserAddress,
                        User.address,
                        tokenAmount
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWith("token amount must be > than zero");
        });

        it("should fail to lock 1155 NFT if caller is not NFT owner", async function () {
            const tokenId = 1;
            const tokenAmount = 1;

            await expect(
                bscBridge.bridge
                    .connect(User)
                    .lock1155(
                        tokenId,
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress,
                        tokenAmount
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWithCustomError(
                mintedCollectionOnBSC,
                "ERC1155MissingApprovalForAll"
            );
        });

        it("should fail to lock 1155 NFT if token id does not exist", async function () {
            const TOKEN_ID_ATTEMPT_TO_LOCK = 5;
            const tokenAmount = 1;

            await expect(
                bscBridge.bridge
                    .connect(bscUser)
                    .lock1155(
                        TOKEN_ID_ATTEMPT_TO_LOCK,
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress,
                        tokenAmount
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWithCustomError(
                mintedCollectionOnBSC,
                "ERC1155InsufficientBalance"
            );
        });

        it("should fail to lock 1155 NFT if token type does not match", async function () {
            const TOKEN_ID_ATTEMPT_TO_LOCK = 5;
            const tokenAmount = 1;
            const { mintedCollectionOnBSCAddress, mintedCollectionOnBSC } =
                await deploy721Collection(bscUser);

            await mintedCollectionOnBSC
                .connect(bscUser)
                .setApprovalForAll(bscBridge.address, true)
                .then((r) => r.wait());

            await expect(
                bscBridge.bridge
                    .connect(bscUser)
                    .lock1155(
                        TOKEN_ID_ATTEMPT_TO_LOCK,
                        DestinationChain,
                        DestinationUserAddress,
                        mintedCollectionOnBSCAddress,
                        tokenAmount
                    )
                    .then((r) => r.wait())
            ).to.be.reverted;
        });
    });

    describe("claimNFT721", async function () {
        let data = {
            tokenId: 2,
            sourceChain: "BSC",
            destinationChain: "ETH",
            destinationUserAddress:
                "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
            sourceNftContractAddress:
                "0xFAd2c09000C681785ECD30C1D0f015Da00E01f7F",
            name: "MyCollection",
            symbol: "MC",
            royalty: 100,
            royaltyReceiver: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
            metadata: "",
            transactionHash:
                "0x5e69043c4d5293ca6d223496c89921e8c7dd906118fbff0dd99a5f8d1ce2aa6a",
            tokenAmount: 1,
            nftType: "singular",
            fee: 5,
        };

        const signatures = [
            "0xf1e57f6831914782247fc9a14d2b9a59330f9a2d9d6cbe55ac6cf0e1b943e8c87956550f707b0b9ea0974f44b3a1a2f795bcb116e82c92e3356f092a0de0404a1b",
            "0xbf3b71e57a36d9ddaf30165700f6c4cca5968dcbae0edb7dfb4a5f70fa6664dd13c79809456d8f3404bfdaacde20775cfa236e9326047a8c49dd5d4d03b3e9f51c",
        ];

        let snapshotId: any;
        beforeEach(async function () {
            // Take a snapshot at the start of each test
            snapshotId = await ethers.provider.send("evm_snapshot", []);
        });
        afterEach(async function () {
            // Revert to the snapshot after each test, effectively resetting any changes
            await ethers.provider.send("evm_revert", [snapshotId]);
        });

        it("should fail to claim 721 if msg.value less than data.fee", async function () {
            await ethUser.sendTransaction({
                to: ethBridge.address,
                value: ethers.parseEther("1"),
            });
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(data, signatures, {
                        value: 1,
                    })
            ).to.be.revertedWith("data.fee LESS THAN sent amount!");
        });

        it("Should fail to claim 721 if current chain is not equal to data.chain", async function () {
            data.destinationChain = "BSC";
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Invalid destination chain!");

            // revert mutation
            data.destinationChain = "ETH";
        });

        it("Should fail to claim 721 if data already processed", async function () {
            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT721(data, signatures, {
                    value: 5,
                });
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Data already processed!");
        });

        it("Should fail to claim 721 if data.nftType is not according to called function", async function () {
            data.nftType = "multiple";
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Invalid NFT type!");

            // revert mutation
            data.nftType = "singular";
        });

        it("Should fail to claim 721 if threshold not reached", async function () {
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT721(data, [signatures[0]], {
                        value: 5,
                    })
            ).to.be.revertedWith("Threshold not reached!");
        });

        it("Should give validators right amount of token on claim (Even)", async function () {
            let [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("0"));
            expect(validator2.pendingReward).to.be.eq(BigInt("0"));
            expect(bridgeBalance).to.be.eq(BigInt("0"));

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT721(data, signatures, {
                    value: 8,
                });

            [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("4"));
            expect(validator2.pendingReward).to.be.eq(BigInt("4"));
            expect(bridgeBalance).to.be.eq(BigInt("8"));
        });

        it("Should give validators right amount of token on claim (Odd)", async function () {
            let [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("0"));
            expect(validator2.pendingReward).to.be.eq(BigInt("0"));
            expect(bridgeBalance).to.be.eq(BigInt("0"));

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT721(data, signatures, {
                    value: 7,
                });

            [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("3"));
            expect(validator2.pendingReward).to.be.eq(BigInt("3"));
            expect(bridgeBalance).to.be.eq(BigInt("7"));
        });
    });

    describe("claimNFT1155", async function () {
        let data = {
            tokenId: 2,
            sourceChain: "BSC",
            destinationChain: "ETH",
            destinationUserAddress:
                "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
            sourceNftContractAddress:
                "0xE3C0bf45000b24D537D9760adf81718fF197D630",
            name: "MyCollection",
            symbol: "MC",
            royalty: 100,
            royaltyReceiver: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
            metadata: "",
            transactionHash:
                "0x6de34e976f3107b37ed992c8aaa6e5f04b3b5a2f127a3256f5b187ad45103911",
            tokenAmount: 1,
            nftType: "multiple",
            fee: 5,
        };
        const signatures = [
            "0x0d43ae15e75d59492c4952bd7465525d16e9970c237d3847d6867fdb4a4a537a525a38ce04e056540ce64fa7ae8aaa7638cd07a5c0fc04ecd78be4e7ac3f99501c",
            "0x204e0cf223bc114ea2785bdf56e1d9e2b10a299e8e5294cb2d525fd1bbb04fbc369a9edac7684c2e5b5df6715d07b69077593391c3483d580bd4dcd979485a811b",
        ];

        let snapshotId: any;
        beforeEach(async function () {
            // Take a snapshot at the start of each test
            snapshotId = await ethers.provider.send("evm_snapshot", []);
        });
        afterEach(async function () {
            // Revert to the snapshot after each test, effectively resetting any changes
            await ethers.provider.send("evm_revert", [snapshotId]);
        });

        it("should fail to claim 1155 if msg.value less than data.fee", async function () {
            await ethUser.sendTransaction({
                to: ethBridge.address,
                value: ethers.parseEther("500"),
            });

            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT1155(data, signatures, {
                        value: 1,
                    })
            ).to.be.revertedWith("data.fee LESS THAN sent amount!");
        });

        it("Should fail to claim 1155 if current chain is not equal to data.chain", async function () {
            data.destinationChain = "BSC";
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT1155(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Invalid destination chain!");

            // revert mutation
            data.destinationChain = "ETH";
        });

        it("Should fail to claim 1155 if data already processed", async function () {
            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT1155(data, signatures, {
                    value: 5,
                });
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT1155(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Data already processed!");
        });

        it("Should fail to claim 1155 if data.nftType is not according to called function", async function () {
            data.nftType = "singular";
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT1155(data, signatures, {
                        value: 5,
                    })
            ).to.be.revertedWith("Invalid NFT type!");

            // revert mutation
            data.nftType = "multiple";
        });

        it("Should fail to claim 1155 if threshold not reached", async function () {
            await expect(
                ethBridge.bridge
                    .connect(ethUser)
                    .claimNFT1155(data, [signatures[0]], {
                        value: 5,
                    })
            ).to.be.revertedWith("Threshold not reached!");
        });

        it("Should give validators right amount of token on claim (Even)", async function () {
            let [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("0"));
            expect(validator2.pendingReward).to.be.eq(BigInt("0"));
            expect(bridgeBalance).to.be.eq(BigInt("0"));

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT1155(data, signatures, {
                    value: 8,
                });

            [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("4"));
            expect(validator2.pendingReward).to.be.eq(BigInt("4"));
            expect(bridgeBalance).to.be.eq(BigInt("8"));
        });

        it("Should give validators right amount of token on claim (Odd)", async function () {
            let [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("0"));
            expect(validator2.pendingReward).to.be.eq(BigInt("0"));
            expect(bridgeBalance).to.be.eq(BigInt("0"));

            await ethBridge.bridge
                .connect(ethUser)
                .claimNFT1155(data, signatures, {
                    value: 7,
                });

            [validator1, validator2, bridgeBalance] = await Promise.all([
                ethBridge.bridge.validators(ethValidator1.address),
                ethBridge.bridge.validators(ethValidator2.address),
                ethers.provider.getBalance(ethBridge.address),
            ]);

            expect(validator1.pendingReward).to.be.eq(BigInt("3"));
            expect(validator2.pendingReward).to.be.eq(BigInt("3"));
            expect(bridgeBalance).to.be.eq(BigInt("7"));
        });
    });

    /* 
        
        
        
        
        
        
        
        
        
        
        
        
        */
    describe("Integration Tests; To and Fro between two chains", async function () {
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
                ] = await lockOnBSCAndClaimOnEth({
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
                });

                await lockOnEthAndClaimOnBSC({
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
                });
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
                ] = await lockOnBSCAndClaimOnEth({
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
                });

                await lockOnEthAndClaimOnBSC({
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
                });
            }
        });

        const getValidatorSignatures_New = async (
            hash: Uint8Array,
            validatorSet: [HardhatEthersSigner, HardhatEthersSigner]
        ): Promise<ReturnType<TGetValidatorSignatures>> => {
            const promises: [Promise<string>, Promise<string>] = [
                validatorSet[0].signMessage(hash),
                validatorSet[1].signMessage(hash),
            ];
            return await Promise.all(promises);
        };

        it.only("should be able to transfer 2 NFTs across multiple chains", async function () {
            const nftType = 721;

            const {
                mintedCollectionOnBSC,
                mintedCollectionOnBSCAddress,
                nftDetails,
                tokenIds,
            } = await deploy721Collection(bscUser);

            const chainIds = ["MOONBEAM", "ARBI", "POLY", "AVAX"];

            let chainArrTemp: TChainArr[] = [
                {
                    bridge: bscBridge,
                    chainId: "BSC",
                    deployer: bscBridgeDeployer,
                    validatorSet: [bscValidator1, bscValidator2],
                    user: bscUser,
                },
                {
                    bridge: ethBridge,
                    chainId: "ETH",
                    deployer: ethBridgeDeployer,
                    validatorSet: [ethValidator1, ethValidator2],
                    user: ethUser,
                },
            ];

            for (const [index, chainId] of chainIds.entries()) {
                const validatorSet: [HardhatEthersSigner, HardhatEthersSigner] =
                    [addrs[index], addrs[index + 1]];

                const validatorAddressSet: [string, string] = [
                    addrs[index].address,
                    addrs[index + 1].address,
                ];

                const deployer = addrs[index + 2];

                const bridge = await deployBridge(
                    chainId,
                    validatorAddressSet,
                    addrs[index + 2]
                );

                chainArrTemp.push({
                    chainId,
                    bridge,
                    validatorSet,
                    deployer,
                    user: addrs[index + 3],
                });
                /*
                    index * 4: Since we're using 4 addresses each time, this computes the next starting address index.
                    + 3: This is added because we're accessing up to addrs[index + 3] within the loop.
                    >= addrs.length - 1: Checks if we're at or have surpassed the last address in addrs.
                */
                if (index * 4 + 3 >= addrs.length - 1) {
                    break;
                }
            }

            console.log({ chainArrTemp });

            const chainArr = chainArrTemp.concat(
                chainArrTemp.slice(0, -1).reverse()
            );

            /* 
                from    ['a', 'b', 'c', 'd', 'e']
                to      ['a', 'b', 'c', 'd', 'e', 'd', 'c', 'b', 'a']
            */

            console.log({ chainArr });

            let lockedEventDatas: TProcessedLogs[] = [],
                duplicateCollectionAddresses: string[] = [],
                duplicateCollectionContracts: Contract[] = [];

            for (const [index] of chainArr.entries()) {
                const source = chainArr[index];
                const destination = chainArr[index + 1];

                console.log("source", source.chainId);
                console.log("destination", destination.chainId);

                if (index === 0) {
                    [
                        lockedEventDatas,
                        duplicateCollectionAddresses,
                        duplicateCollectionContracts,
                    ] = await lockOnBSCAndClaimOnEth({
                        mintedCollectionOnBSC,
                        tokenIds,
                        mintedCollectionOnBSCAddress,
                        nftDetails,
                        bscUser: source.user, //           source user
                        bscBridge: source.bridge, //       source bridge
                        ethUser: destination.user, //       destination user
                        ethBridge: destination.bridge, //   destination bridge
                        nftType,
                        getValidatorSignatures,
                    });
                } else {
                    console.log("CASE 2", index);

                    let lockOnEthReceipt1: ContractTransactionReceipt | null,
                        lockOnEthReceipt2: ContractTransactionReceipt | null;

                    [lockedEventDatas, lockOnEthReceipt1, lockOnEthReceipt2] =
                        await lock({
                            lockedEventDatas,
                            duplicateCollectionContracts:
                                duplicateCollectionContracts as any,
                            duplicateCollectionAddresses,
                            nftDetails,
                            sourceUser: source.user,
                            sourceBridge: source.bridge,
                            destinationUser: destination.user,
                            destinationBridge: destination.bridge,
                            nftType,
                        });

                    [
                        duplicateCollectionAddresses,
                        duplicateCollectionContracts,
                    ] = await claim({
                        lockedOnEthLogData1: lockedEventDatas[0],
                        lockedOnEthLogData2: lockedEventDatas[1],
                        lockOnEthReceipt1,
                        lockOnEthReceipt2,
                        mintedCollectionOnBSC,
                        mintedCollectionOnBSCAddress,
                        nftDetails,
                        destinationUser: destination.user,
                        destinationBridge: destination.bridge,
                        sourceUser: source.user,
                        nftType,
                        getValidatorSignatures: async (hash: Uint8Array) =>
                            getValidatorSignatures_New(
                                hash,
                                destination.validatorSet
                            ),
                    });

                    // ==============================================================
                    if (index + 1 === chainArr.length - 1) {
                        break;
                    }
                }
                console.log(
                    "================================================="
                );
            }
        });
    });
});
