import { ethers } from "hardhat";

async function main() {

  const sdf = await ethers.getContractFactory("HederaNFTStorageDeployer");
  const storageDeployerContract = await sdf.deploy();
  const sd = await storageDeployerContract.waitForDeployment();

  console.log(
    `StorageDeployerContract contract deployed at ${await sd.getAddress()}`
  );

  const bootstrapValidator = ["0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"];
  const chainSymbol = "HEDERA";
  await new Promise((r) => setTimeout(r, 10000));
  const bf = await ethers.getContractFactory("HederaBridge");
  const bridge = await bf.deploy(
    bootstrapValidator,
    chainSymbol,
    sd,
    {
      gasLimit: 9500000,
    }
  );
  console.log(`Bridge contract deployed at ${await bridge.getAddress()}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
