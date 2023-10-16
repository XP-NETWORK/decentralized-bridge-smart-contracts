import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";

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
    return { bridge };
  };

  const mintNft = async (address: string) => {
    const Bridge = await ethers.getContractFactory("Bridge");

  }

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

  it("lock nft", async () => {

  });

  it("unlock nft", async () => {

  });
});
