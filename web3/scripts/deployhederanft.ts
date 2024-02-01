import { ethers } from "hardhat";

async function main() {
  const nftf = await ethers.getContractFactory("XPNftHts");
  const nft = await nftf.deploy(
    "bruh",
    "BRUH",
    1,
    "0x299f21507aC64fa403dc2E0E2B0fa2CC580aF419",
    {
      value: "50000000000000000000",
      gasLimit: "5000000",
    }
  );

  await nft.deploymentTransaction();
  console.log("NFT deployed to:", await nft.getAddress());
  console.log(
    "Mint: ",
    await nft["mint(address,uint256,uint256,address,string)"](
      "0x299f21507aC64fa403dc2E0E2B0fa2CC580aF419",
      "1",
      "0",
      "0x299f21507aC64fa403dc2E0E2B0fa2CC580aF419",
      "https://gat"
    )
  );

  console.log();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
