import { ethers } from "hardhat";
import { ContractProxy } from "../contractsTypes";
import { EventLog } from "ethers";

async function main() {
    console.log((await ethers.getSigners())[0].address)
  const nftf = await ethers.getContractFactory("ContractProxy");
  const nft = nftf.attach("0xFdDEEbDf5F2e959A1637Cb130cE753d42083a2EA") as ContractProxy;
  const mint = await nft.deployNft("TESTTOKEN", "TESTTKN", {
    value: ethers.parseEther("50"),
    gasLimit: 15000000
  })
  console.log(mint)
  const receipt = await mint.wait()
  const ev = receipt?.logs.find((e) => {
    if (e instanceof EventLog) {
        const a = e.eventName === "NftCollectionCreated";
        return a
    } else {
        return false
    }
  })
  const address = (ev as EventLog).args[0];
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
