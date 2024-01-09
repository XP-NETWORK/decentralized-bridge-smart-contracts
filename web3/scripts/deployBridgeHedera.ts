import { ethers } from "hardhat";

async function main() {
  const cdfa = await ethers.getContractFactory("HederaCollectionDeployer");
  const cdd = await cdfa.deploy();
  const cd = await cdd.waitForDeployment();
  console.log(
    `NFTCollectionDeployer contract deployed at ${await cd.getAddress()}`
  );

  const sdf = await ethers.getContractFactory("HederaNFTStorageDeployer");
  const storageDeployerContract = await sdf.deploy();
  const sd = await storageDeployerContract.waitForDeployment();

  console.log(
    `StorageDeployerContract contract deployed at ${await sd.getAddress()}`
  );

  const bootstrapValidator = ["0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"];
  const chainSymbol = "HBAR";
  const bf = await ethers.getContractFactory("HederaBridge");
  const bridge = await bf.deploy(
    bootstrapValidator,
    chainSymbol,
    await cd.getAddress(),
    await sd.getAddress()
  );
  console.log(`Bridge contract deployed at ${await bridge.getAddress()}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
