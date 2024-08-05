import { ethers } from "hardhat";

async function main() {
  // const sdf = await ethers.getContractFactory("HederaNFTStorageDeployer");
  // const storageDeployerContract = await sdf.deploy();
  // const sd = await storageDeployerContract.waitForDeployment();

  // console.log(
  //   `StorageDeployerContract contract deployed at ${await sd.getAddress()}`
  // );


  // return
  
  let res = false;
  while (!res) {
    try {
      const bootstrapValidator = ["0xe7D463DFf4E8c01040DafD137598d006292A7Aa3"];
      const chainSymbol = "HEDERA";
      await new Promise((r) => setTimeout(r, 5000));
      const bf = await ethers.getContractFactory("HederaBridge");
      const bridge = await bf.deploy(
        bootstrapValidator,
        chainSymbol,
        "0xbFDdaF774a3690a44852c70694E05cDaeadae792",
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
