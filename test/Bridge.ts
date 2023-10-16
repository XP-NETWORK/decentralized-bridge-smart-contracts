import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { use } from "chai";

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
    console.log(await bridge.returnValidators());
    console.log((await bridge.returnValidators()).length);
  });

  it("add new validator", async () => {
    const [newValidator] = await ethers.getSigners();
    const { bridge } = await loadFixture(prepareAll);
    await bridge.addNewValidator(newValidator.address);
    console.log((await bridge.returnValidators()).length);
  });

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
    nftWait.wait();
    let contractAddress = await nft.getAddress();

    let waitProve = await nft
      .connect(user1)
      .approve(bridge.getAddress(), BigInt("0"));
    waitProve.wait();

    let alex = await bridge
      .connect(user1)
      .lock(contractAddress, user1.address, BigInt(0), "ETH");
  });

  it("unlock nft", async () => {});
});
