import { ethers } from "hardhat";

async function main() {
  let res = false;

  while (!res) {
    try {
      await new Promise((r) => setTimeout(r, 2000));
      const nftf = await ethers.getContractFactory("ContractProxy");
      const nft = await nftf.deploy({
        // gasLimit: 9500000,
        gasLimit: 15000000,
      });
      await nft.waitForDeployment();
      console.log("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAa");
      res = true;
      break;
      console.log(await nft.getAddress())
    }
    catch (ex) {
      console.log("error", ex);
      await new Promise((r) => setTimeout(r, 2000));

    }
  }

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
