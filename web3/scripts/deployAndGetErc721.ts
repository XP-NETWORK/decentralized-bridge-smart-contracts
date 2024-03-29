import { ethers } from "hardhat";

async function main() {


  const nft721 = await ethers.deployContract("ERC721Royalty", ["DUM", "D", "0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"]);

  await nft721.waitForDeployment();
  const tx = nft721.mint("0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23", 69, 0, "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23", "abc.com/1");
  await (await tx).wait()
  console.log(
    ` deployed to ${nft721.target}, ${(await tx).hash}`
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
