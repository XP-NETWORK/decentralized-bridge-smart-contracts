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

    const bootstrapValidator = ["0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"]
    const chainSymbol = "ETH"
    const bridge = await ethers.deployContract("Bridge", [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress]);

    await bridge.waitForDeployment();

    console.log(
        `Bridge contract deployed at ${bridge.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
