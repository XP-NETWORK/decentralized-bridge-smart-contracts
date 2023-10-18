import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { use } from "chai";
import { keccak256 } from "ethers";

import { ethers } from "hardhat";

describe("Bridge", () => {
  const prepareAll = async () => {
    const [
      validtor1,
      validtor2,
      validtor3,
      validtor4,
      validtor5,
      validtor6,
      validtor7,
      validtor8,
      validtor9,
      validtor10,
      validtor11,
    ] = await ethers.getSigners();
    let arrayOfValidators = [
      validtor1.address,
      validtor2.address,
      validtor3.address,
      validtor4.address,
      validtor5.address,
      validtor6.address,
      validtor7.address,
      validtor8.address,
      validtor9.address,
      validtor10.address,
      validtor11.address,
    ];
    const Bridge = await ethers.getContractFactory("Bridge");
    const bridge = await Bridge.deploy(arrayOfValidators);
    await bridge.waitForDeployment();
    return { bridge };
  };

  const deployNFTerc721Contract = async () => {
    const Nft = await ethers.getContractFactory("TestNFTerc721");
    const nft = await Nft.deploy(
      "nft",
      "nft",
      "https://ipfs.io/ipfs/QmNgSudWimtho9aE6v49CfkoV3dmJFXLLLB2XwVopZ2Hp6"
    );
    nft.waitForDeployment();
    const [user1, user2, user3] = await ethers.getSigners();
    return { nft, user1, user2, user3 };
  };

  it("init contract and check validators", async () => {
    const { bridge } = await loadFixture(prepareAll);
  });

  it("add new validator", async () => {
    const [newValidator] = await ethers.getSigners();
    const { bridge } = await loadFixture(prepareAll);
    await bridge.addNewValidator(newValidator.address);
    // console.log(await bridge.returnValidators().length);
  });

  it("should add validator and sign", async () => {
    const { bridge } = await loadFixture(prepareAll);
    const [user1, user2, user3] = await ethers.getSigners();
    const signature = await user1.signMessage("hello");

    const sig = await user1.signMessage(signature);

    let alex = await bridge
      .connect(user2)
      .validatorSignature(keccak256(signature), sig);

      console.log(alex,user1.address);

  });
  /*

  it("mint nft", async () => {
    const { nft, user1 } = await loadFixture(deployNFTerc721Contract);
    await nft.mint(user1.address);
    console.log(await nft.balanceOf(user1.address));
    console.log(await nft.ownerOf("0"));
    console.log(await nft.tokenURI("0"));
  });

  it("lock nft", async () => {
    const { bridge } = await loadFixture(prepareAll);
    const { nft, user1 } = await loadFixture(deployNFTerc721Contract);

    let nftWait = await nft.mint(user1.address);
    await nftWait.wait();
    let contractAddress = await nft.getAddress();
    let waitProve = await nft
      .connect(user1)
      .approve(bridge.getAddress(), BigInt("0"));
    await waitProve.wait();

    let waitTransfer = await bridge
      .connect(user1)
      .lock(
        BigInt(0),
        "ETH",
        user1.address,
        contractAddress,
        "BSC",
        contractAddress.toString(),
        user1.address,
        "https://ipfs.io/ipfs/QmNgSudWimtho9aE6v49CfkoV3dmJFXLLLB2XwVopZ2Hp6",
        "123"
      );
    await waitTransfer.wait();
    console.log(
      { contractAddress },
      "new owner: ",
      await nft.ownerOf("0"),
      "user1: ",
      user1.address,
      "bridge adress: ",
      await bridge.getAddress()
    );
  });

  it("unlock nft", async () => {
    const { bridge } = await loadFixture(prepareAll);
    const { nft, user1 } = await loadFixture(deployNFTerc721Contract);

    let nftWait = await nft.mint(user1.address);
    await nftWait.wait();
    let contractAddress = await nft.getAddress();
    let waitProve = await nft
      .connect(user1)
      .approve(bridge.getAddress(), BigInt("0"));
    await waitProve.wait();

    let waitTransfer = await bridge
      .connect(user1)
      .lock(
        BigInt(0),
        "ETH",
        user1.address,
        contractAddress,
        "BSC",
        contractAddress.toString(),
        user1.address,
        "https://ipfs.io/ipfs/QmNgSudWimtho9aE6v49CfkoV3dmJFXLLLB2XwVopZ2Hp6",
        "123"
      );
    await waitTransfer.wait();
    console.log(
      { contractAddress },
      "new owner: ",
      await nft.ownerOf("0"),
      "user1: ",
      user1.address,
      "bridge adress: ",
      await bridge.getAddress()
    );
    let waitUnlock = await bridge
      .connect(user1)
      .unLock(user1.address, BigInt(0), contractAddress);

    await waitUnlock.wait();
    console.log(
      { contractAddress },
      "new owner: ",
      await nft.ownerOf("0"),
      "user1: ",
      user1.address,
      "bridge adress: ",
      await bridge.getAddress()
    );
  });
  */
});
