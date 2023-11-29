import { task } from "hardhat/config";

task("deployBridge", "Deploys the Bridge contract")
    .addParam("chainSymbol", "The chain symbol")
    .addParam("bootstrapValidatorAddress", "The chain symbol")
    .setAction(async (taskArgs, hre) => {
        const ethers = hre.ethers;
        const collectionDeployerContract = await ethers.deployContract("NFTCollectionDeployer");
        await collectionDeployerContract.waitForDeployment();
        const collectionDeployerContractAddress = collectionDeployerContract.target
        const storageDeployerContract = await ethers.deployContract("NFTStorageDeployer");
        await storageDeployerContract.waitForDeployment();
        const storageDeployerContractAddress = storageDeployerContract.target
        const { chainSymbol, bootstrapValidatorAddress } = taskArgs;
        const bootstrapValidator = [bootstrapValidatorAddress]
        console.log(chainSymbol)
        const bridge = await ethers.deployContract("Bridge", [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress]);

        await bridge.waitForDeployment();

        console.log(
            `Bridge contract deployed at ${bridge.target} -${chainSymbol}`
        );
    })