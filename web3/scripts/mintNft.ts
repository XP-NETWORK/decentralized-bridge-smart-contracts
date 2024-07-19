import { ethers } from "hardhat";
import { ContractProxy, IHRC, IHRC__factory } from "../contractsTypes";

async function main() {
  const nftf = await ethers.getContractFactory("ContractProxy");
  const nft = nftf.attach(
    "0xFdDEEbDf5F2e959A1637Cb130cE753d42083a2EA"
  ) as ContractProxy;
  const ihrc = await ethers.getContractAt("lib/hedera/contracts/hts-precompile/IHRC.sol:IHRC", "0x0000000000000000000000000000000000426f9d")
  const assocaited = await ihrc.associate()
  const mint = await nft.mint('0x0000000000000000000000000000000000426f9d', 'https://meta.polkamon.com/meta?id=10001852306', {
    value: ethers.parseEther("10"),
    gasLimit:  15000000
  })
  console.log(await mint.wait())
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
