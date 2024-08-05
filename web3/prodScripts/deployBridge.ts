import { task } from "hardhat/config";

task("deployBridge", "Deploys the Bridge contract")
    .addParam("chainSymbol", "The chain symbol")
    .addParam("bootstrapValidatorAddress", "The chain symbol")
    .setAction(async (taskArgs, hre) => {
        const ethers = hre.ethers;
        const upgrades = hre.upgrades;
        const collectionDeployerContract = await ethers.deployContract("NFTCollectionDeployer");
        await collectionDeployerContract.waitForDeployment();
        const collectionDeployerContractAddress = collectionDeployerContract.target
        const storageDeployerContract = await ethers.deployContract("NFTStorageDeployer");
        await storageDeployerContract.waitForDeployment();
        const storageDeployerContractAddress = storageDeployerContract.target
        const { chainSymbol, bootstrapValidatorAddress } = taskArgs;
        const bootstrapValidator = [bootstrapValidatorAddress]
        console.log(chainSymbol)
        const bridge = await upgrades.deployProxy(await ethers.getContractFactory("Bridge"), [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress, "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"]);

        await bridge.waitForDeployment();

        console.log(
            `Bridge contract deployed at ${bridge.target} -${chainSymbol}`
        );
    })