import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import {
  Contract,
  ContractTransactionReceipt,
  EventLog,
  Typed,
  ZeroAddress,
  keccak256,
} from "ethers";
import { ethers } from "hardhat";
import {
  Bridge__factory,
  ERC1155Royalty,
  ERC721Royalty,
  HederaBridge__factory,
} from "../../contractsTypes";
import {
  TChainArrWithBridge,
  TGetValidatorSignatures,
  THederaBridge,
  THederaChainArr,
  TProcessedLogs,
} from "./types";
import {
  NftTransferDetailsTypes,
  claim,
  deploy1155Collection,
  deploy721Collection,
  encoder,
  formatSignatures,
  hexStringToByteArray,
  lock,
  lockOnBSCAndClaimOnEth,
  lockOnEthAndClaimOnBSC,
} from "./utils";

describe("Bridge", function () {
  let Bridge: HederaBridge__factory,
    bscBridge: THederaBridge,
    ethBridge: THederaBridge;

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
      "HederaCollectionDeployer"
    );
    const collectionInstance = await CollectionDeployer.connect(
      deployer
    ).deploy();
    const collectionDeployer = await collectionInstance
      .connect(deployer)
      .getAddress();

    const StorageDeployer = await ethers.getContractFactory(
      "HederaNFTStorageDeployer"
    );
    const storageDeployerInstance = await StorageDeployer.connect(
      deployer
    ).deploy();
    const storageDeployer = await storageDeployerInstance
      .connect(deployer)
      .getAddress();

    Bridge = await ethers.getContractFactory("HederaBridge");

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
        expect((await bscBridge.bridge.validators(validator)).added).to.equal(
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

  describe("claimValidatorRewards", async function () {
    const createClaimValidatorHash = async (validatorAddress: string) => {
      const hash = keccak256(encoder.encode(["address"], [validatorAddress]));
      const hexifiedDataHash = hexStringToByteArray(hash);
      const signatures = await getValidatorSignatures(hexifiedDataHash, "eth");
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
      const signatures = await createClaimValidatorHash(ethValidator1.address);

      await expect(
        ethBridge.bridge
          .claimValidatorRewards(ethValidator1.address, [signatures[0]])
          .then((r) => r.wait())
      ).to.be.revertedWith("Threshold not reached!");
    });

    // it("should successfully transfer funds when validator claims rewards", async function () {
    //   let data = {
    //     tokenId: 2,
    //     sourceChain: "BSC",
    //     destinationChain: "ETH",
    //     destinationUserAddress: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
    //     sourceNftContractAddress: "0x2c9375211b1a5fd1fb44b2bd2020b368b6ec8aa7",
    //     name: "MyCollection",
    //     symbol: "MC",
    //     royalty: 100,
    //     royaltyReceiver: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
    //     metadata: "",
    //     transactionHash:
    //       "0xc1bbb33b2192025ccfe64dbc96a8ab3b9e7bcbf991a74e8d896ba533cb58caad",
    //     tokenAmount: 1,
    //     nftType: "multiple",
    //     fee: 5,
    //   };
    //   const signatures = [
    //     "0x33aad697034440681159f08b64acc740a5db9a22cc2a851ae99e239b601f89993162154f57e0278901e13e33435b38491d0a33cefc6083e61e5c77abf5d127b21c",
    //     "0x9bc0a62f184035c0ca7b421434400f21b9e4cabe57ba45224914e98fd6457d0e34b5ab0e239227628ea0cbf9df6aeeee796f3f43991f606cadafcdc8c4eaea191b",
    //   ];

    //   let [validator1, bridgeBalance, validator_1_balance] = await Promise.all([
    //     ethBridge.bridge.validators(ethValidator1.address),
    //     ethers.provider.getBalance(ethBridge.address),
    //     ethers.provider.getBalance(ethValidator1.address),
    //   ]);

    //   expect(validator1.pendingReward).to.be.eq(BigInt("0"));
    //   expect(bridgeBalance).to.be.eq(BigInt("0"));
    //   expect(validator_1_balance).to.be.eq(BigInt("10000000000000000000000"));

    //   await ethBridge.bridge.connect(ethUser).claimNFT1155(data, signatures, {
    //     value: 8,
    //   });

    //   [validator1, bridgeBalance] = await Promise.all([
    //     ethBridge.bridge.validators(ethValidator1.address),
    //     ethers.provider.getBalance(ethBridge.address),
    //   ]);

    //   expect(validator1.pendingReward).to.be.eq(BigInt("4"));
    //   expect(bridgeBalance).to.be.eq(BigInt("8"));

    //   const claimSignatures = await createClaimValidatorHash(
    //     ethValidator1.address
    //   );
    //   await ethBridge.bridge
    //     .claimValidatorRewards(ethValidator1.address, claimSignatures)
    //     .then((r) => r.wait());

    //   validator_1_balance = await ethers.provider.getBalance(
    //     ethValidator1.address
    //   );

    //   expect(validator_1_balance).to.be.eq(BigInt("10000000000000000000004"));
    // });
  });

  describe("addValidator", async function () {
    const createAddValidatorHash = async (validatorAddress: string) => {
      const hash = keccak256(encoder.encode(["address"], [validatorAddress]));
      const hexifiedDataHash = hexStringToByteArray(hash);
      const signatures = await getValidatorSignatures(hexifiedDataHash, "bsc");
      return signatures;
    };
    it("Should fail if zero address for validator is provided", async function () {
      const signatures = await createAddValidatorHash(ZeroAddress);
      await expect(
        bscBridge.bridge
          .addValidator(ZeroAddress, formatSignatures(signatures))
          .then((r) => r.wait())
      ).to.be.revertedWith("Address cannot be zero address!");
    });

    it("Should fail if no signatures are provided", async function () {
      const newValidator = addrs[10];

      await expect(
        bscBridge.bridge.addValidator(newValidator, []).then((r) => r.wait())
      ).to.be.revertedWith("Must have signatures!");
    });

    it("Should fail if validators do not reach threshold", async function () {
      const newValidator = addrs[10];
      const signatures = await createAddValidatorHash(newValidator.address);

      await expect(
        bscBridge.bridge
          .addValidator(newValidator.address, [
            { signature: signatures[0], signerAddress: "" },
          ])
          .then((r) => r.wait())
      ).to.be.revertedWith("Threshold not reached!");
    });

    it("Should successfully add a new validator with correct arguments", async function () {
      const newValidator = addrs[10];

      const [signatures, beforeValidatorAdditionCount] = await Promise.all([
        createAddValidatorHash(newValidator.address),
        bscBridge.bridge.validatorsCount(),
      ]);

      const formattedSignatures = signatures.map((sig) => ({
        signature: sig,
        signerAddress: "",
      }));

      const receipt = await bscBridge.bridge
        .addValidator(newValidator.address, formattedSignatures)
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
      ).to.be.revertedWith("sourceNftContractAddress cannot be zero address");
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

  describe("claimNFT721", async function () {
    let data = {
      tokenId: 2,
      sourceChain: "BSC",
      destinationChain: "ETH",
      destinationUserAddress: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
      sourceNftContractAddress: "0xba92cf00f301b9fa4cf5ead497d128bdb3e05e1b",
      name: "MyCollection",
      symbol: "MC",
      royalty: 100,
      royaltyReceiver: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
      metadata: "",
      transactionHash:
        "0x9724e4d237117018e5d2135036d879b25ca36ae4469120b85ef7ebba8fa408d5",
      tokenAmount: 1,
      nftType: "singular",
      fee: 5,
    };

    const signatures = [
      "0x90d2b34877bbabe4a6bc401f10db04a3debdfddc2316a1ddfda111968098feec55f6d01bf380e8191f514dd1c7175c461a4849b791389e39217e2bd69d946ec41b",
      "0x552e322d425e6daba13edfb6c205b07de52a8ce2a40f80be919e582da642d508696c26cd7659c637b812bbb88a9ee69f3d32b9a3abbfabb779706a3f5cbd74bb1c",
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
        ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
          value: 1,
        })
      ).to.be.revertedWith("data.fee LESS THAN sent amount!");
    });

    it("Should fail to claim 721 if current chain is not equal to data.chain", async function () {
      data.destinationChain = "BSC";
      await expect(
        ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
          value: 5,
        })
      ).to.be.revertedWith("Invalid destination chain!");

      // revert mutation
      data.destinationChain = "ETH";
    });

    it("Should fail to claim 721 if data already processed", async function () {
      await ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
        value: 5,
      });
      await expect(
        ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
          value: 5,
        })
      ).to.be.revertedWith("Data already processed!");
    });

    it("Should fail to claim 721 if data.nftType is not according to called function", async function () {
      data.nftType = "multiple";
      await expect(
        ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
          value: 5,
        })
      ).to.be.revertedWith("Invalid NFT type!");

      // revert mutation
      data.nftType = "singular";
    });

    it("Should fail to claim 721 if threshold not reached", async function () {
      await expect(
        ethBridge.bridge.connect(ethUser).claimNFT721(data, [signatures[0]], {
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

      await ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
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

      await ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
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

    it("should be able to claim and lock a 721 NFT from a non EVM source", async function () {
      const oldSourceNftContractAddress = data.sourceNftContractAddress;

      // a dummy non evm address (Elrond)
      data.sourceNftContractAddress =
        "erd1m229kx85t9jsamjuxpu6sjtu6jws7q4lesne9m5gdex9g8ps6n9scwk2V0";

      const nftTransferDetailsValues = Object.values(data);

      const dataHash = keccak256(
        encoder.encode(NftTransferDetailsTypes, nftTransferDetailsValues)
      );

      const hexifiedDataHash = hexStringToByteArray(dataHash);
      const signatures = await getValidatorSignatures(hexifiedDataHash, "eth");

      await ethBridge.bridge.connect(ethUser).claimNFT721(data, signatures, {
        value: 5,
      });

      const duplicate = await ethBridge.bridge.originalToDuplicateMapping(
        data.sourceNftContractAddress,
        data.sourceChain
      );
      const original = await ethBridge.bridge.duplicateToOriginalMapping(
        duplicate[1],
        duplicate[0]
      );

      expect(duplicate[1]).to.not.be.eq("");
      expect(original[1]).to.be.eq(data.sourceNftContractAddress);

      const duplicateCollectionContract = await ethers.getContractAt(
        "ERC721Royalty",
        duplicate[1]
      );
      await duplicateCollectionContract
        .connect(ethUser)
        .approve(ethBridge.address, data.tokenId);

      const receipt = await ethBridge.bridge
        .connect(ethUser)
        .lock721(data.tokenId, data.sourceChain, bscUser.address, duplicate[1])
        .then((r) => r.wait());

      const logs = receipt?.logs[1] as EventLog;
      expect(logs.args[3]).to.be.eq(data.sourceNftContractAddress);

      // revert the mutation done at the beginning of the test
      data.sourceNftContractAddress = oldSourceNftContractAddress;
    });
  });

  describe("Integration Tests; To and Fro between two chains", async function () {
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

    const _getValidatorSignatures = async (
      hash: Uint8Array,
      validatorSet: [HardhatEthersSigner, HardhatEthersSigner]
    ): Promise<ReturnType<TGetValidatorSignatures>> => {
      const promises: [Promise<string>, Promise<string>] = [
        validatorSet[0].signMessage(hash),
        validatorSet[1].signMessage(hash),
      ];
      return await Promise.all(promises);
    };

    it("should be able to transfer 2 NFTs across multiple chains", async function () {
      const nftType = 721;

      const {
        mintedCollectionOnBSC,
        mintedCollectionOnBSCAddress,
        nftDetails,
        tokenIds,
      } = await deploy721Collection(bscUser);

      let chainArr: THederaChainArr[] = [
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
        {
          bridge: null,
          chainId: "MOONBEAM",
          deployer: addrs[0],
          validatorSet: [addrs[1], addrs[2]],
          user: addrs[3],
        },
        {
          bridge: null,
          chainId: "ARBI",
          deployer: addrs[4],
          validatorSet: [addrs[5], addrs[6]],
          user: addrs[7],
        },
        {
          bridge: null,
          chainId: "POLY",
          deployer: addrs[8],
          validatorSet: [addrs[9], addrs[10]],
          user: addrs[11],
        },
        {
          bridge: null,
          chainId: "ARBI",
          deployer: addrs[4],
          validatorSet: [addrs[5], addrs[6]],
          user: addrs[7],
        },
        {
          bridge: null,
          chainId: "MOONBEAM",
          deployer: addrs[0],
          validatorSet: [addrs[1], addrs[2]],
          user: addrs[3],
        },
        {
          bridge: ethBridge,
          chainId: "ETH",
          deployer: ethBridgeDeployer,
          validatorSet: [ethValidator1, ethValidator2],
          user: ethUser,
        },
        {
          bridge: bscBridge,
          chainId: "BSC",
          deployer: bscBridgeDeployer,
          validatorSet: [bscValidator1, bscValidator2],
          user: bscUser,
        },
      ];

      for (const [index, chain] of chainArr.entries()) {
        if (chain.bridge !== null) continue;

        const validatorAddressSet: [string, string] = [
          chain.validatorSet[0].address,
          chain.validatorSet[1].address,
        ];

        const bridge = await deployBridge(
          chain.chainId,
          validatorAddressSet,
          chain.deployer
        );

        chainArr[index].bridge = bridge;
      }

      let lockedEventDatas: TProcessedLogs[] = [],
        duplicateCollectionAddresses: string[] = [],
        duplicateCollectionContracts: Contract[] = [];

      for (const [index] of chainArr.entries()) {
        const source = chainArr[index] as TChainArrWithBridge;
        const destination = chainArr[index + 1] as TChainArrWithBridge;

        if (!source.bridge || !destination.bridge) {
          throw new Error(`A bridge at index ${index} is null`);
        }

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
          let lockOnEthReceipt1: ContractTransactionReceipt | null,
            lockOnEthReceipt2: ContractTransactionReceipt | null;

          [lockedEventDatas, lockOnEthReceipt1, lockOnEthReceipt2] = await lock(
            {
              lockedEventDatas,
              duplicateCollectionContracts: duplicateCollectionContracts as any,
              duplicateCollectionAddresses,
              nftDetails,
              source,
              destination,
              sourceUser: source.user,
              destinationUser: destination.user,
              nftType,
            }
          );

          [duplicateCollectionAddresses, duplicateCollectionContracts] =
            await claim({
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
                _getValidatorSignatures(hash, destination.validatorSet),
            });

          // ==============================================================
          if (index + 1 === chainArr.length - 1) {
            break;
          }
        }
      }
    });
    it("should be able to transfer 2 NFTs across multiple chains", async function () {
      const nftType = 1155;

      const {
        mintedCollectionOnBSC,
        mintedCollectionOnBSCAddress,
        nftDetails,
        tokenIds,
      } = await deploy1155Collection(2, bscUser);

      let chainArr: THederaChainArr[] = [
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
        {
          bridge: null,
          chainId: "MOONBEAM",
          deployer: addrs[0],
          validatorSet: [addrs[1], addrs[2]],
          user: addrs[3],
        },
        {
          bridge: null,
          chainId: "ARBI",
          deployer: addrs[4],
          validatorSet: [addrs[5], addrs[6]],
          user: addrs[7],
        },
        {
          bridge: null,
          chainId: "POLY",
          deployer: addrs[8],
          validatorSet: [addrs[9], addrs[10]],
          user: addrs[11],
        },
        {
          bridge: null,
          chainId: "ARBI",
          deployer: addrs[4],
          validatorSet: [addrs[5], addrs[6]],
          user: addrs[7],
        },
        {
          bridge: null,
          chainId: "MOONBEAM",
          deployer: addrs[0],
          validatorSet: [addrs[1], addrs[2]],
          user: addrs[3],
        },
        {
          bridge: ethBridge,
          chainId: "ETH",
          deployer: ethBridgeDeployer,
          validatorSet: [ethValidator1, ethValidator2],
          user: ethUser,
        },
        {
          bridge: bscBridge,
          chainId: "BSC",
          deployer: bscBridgeDeployer,
          validatorSet: [bscValidator1, bscValidator2],
          user: bscUser,
        },
      ];

      for (const [index, chain] of chainArr.entries()) {
        if (chain.bridge !== null) continue;

        const validatorAddressSet: [string, string] = [
          chain.validatorSet[0].address,
          chain.validatorSet[1].address,
        ];

        const bridge = await deployBridge(
          chain.chainId,
          validatorAddressSet,
          chain.deployer
        );

        chainArr[index].bridge = bridge;
      }

      let lockedEventDatas: TProcessedLogs[] = [],
        duplicateCollectionAddresses: string[] = [],
        duplicateCollectionContracts: Contract[] = [];

      for (const [index] of chainArr.entries()) {
        const source = chainArr[index] as TChainArrWithBridge;
        const destination = chainArr[index + 1] as TChainArrWithBridge;

        if (!source.bridge || !destination.bridge) {
          throw new Error(`A bridge at index ${index} is null`);
        }

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
          let lockOnEthReceipt1: ContractTransactionReceipt | null,
            lockOnEthReceipt2: ContractTransactionReceipt | null;

          [lockedEventDatas, lockOnEthReceipt1, lockOnEthReceipt2] = await lock(
            {
              lockedEventDatas,
              duplicateCollectionContracts: duplicateCollectionContracts as any,
              duplicateCollectionAddresses,
              nftDetails,
              source,
              destination,
              sourceUser: source.user,
              destinationUser: destination.user,
              nftType,
            }
          );

          [duplicateCollectionAddresses, duplicateCollectionContracts] =
            await claim({
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
                _getValidatorSignatures(hash, destination.validatorSet),
            });

          if (index + 1 === chainArr.length - 1) {
            break;
          }
        }
      }
    });
  });
});
