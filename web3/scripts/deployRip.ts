import { ethers } from "hardhat";

async function main() {
  const nftf = await ethers.getContractFactory("ContractProxy");
  const nft = await nftf.deploy();

  await nft.waitForDeployment();
  console.log(await nft.getAddress())
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
