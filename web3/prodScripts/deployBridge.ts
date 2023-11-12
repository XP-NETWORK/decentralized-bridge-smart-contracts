import { ethers } from "hardhat";

async function main() {

    const collectionDeployerContract = await ethers.deployContract("NFTCollectionDeployer");
    await collectionDeployerContract.waitForDeployment();
    const collectionDeployerContractAddress = collectionDeployerContract.target
    console.log(
        `NFTCollectionDeployer contract deployed at ${collectionDeployerContractAddress}`
    );


    const storageDeployerContract = await ethers.deployContract("NFTStorageDeployer");
    await storageDeployerContract.waitForDeployment();
    const storageDeployerContractAddress = storageDeployerContract.target
    console.log(
        `StorageDeployerContract contract deployed at ${storageDeployerContractAddress}`
    );

    const bootstrapValidator = ["0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"]
    const chainSymbol = "BSC"
    const bridge = await ethers.deployContract("Bridge", [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress]);

    await bridge.waitForDeployment();

    console.log(
        `BridgeStorage contract deployed at ${bridge.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});