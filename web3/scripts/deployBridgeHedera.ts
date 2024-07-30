import { ethers } from "hardhat";

async function main() {
  // const sdf = await ethers.getContractFactory("HederaNFTStorageDeployer");
  // const storageDeployerContract = await sdf.deploy();
  // const sd = await storageDeployerContract.waitForDeployment();

  // console.log(
  //   `StorageDeployerContract contract deployed at ${await sd.getAddress()}`
  // );

  
  let res = false;
  while (!res) {
    try {
      const bootstrapValidator = ["0xe9dFDea6Da1Dc1906027f5412Dc137E7D9E2A9e4"];
      const chainSymbol = "HEDERA";
      await new Promise((r) => setTimeout(r, 2000));
      const bf = await ethers.getContractFactory("HederaBridge");
      const bridge = await bf.deploy(
        bootstrapValidator,
        chainSymbol,
        "0xb11aa122633A8104B1A655C3a12e296e8634926c",
        {
          // gasLimit: 9500000,
          gasLimit: 15000000,
        }
      );
      res = true;
      console.log(`Bridge contract deployed at ${await bridge.getAddress()}`);
    }
    catch (ex) {
      console.log("error",ex);
      res = false;
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
