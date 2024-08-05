import { ContractFactory, keccak256 } from "ethers";
import { ethers, upgrades } from "hardhat";
import { encoder, hexStringToByteArray } from "../test/utils";
import { UpgradeProxyOptions } from "@openzeppelin/hardhat-upgrades/dist/utils";


async function main() {
    const [signer] = await ethers.getSigners();

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

    const bootstrapValidator = [signer.address]
    const chainSymbol = "ETH"
    const contractFactory1 = await ethers.getContractFactory("Bridge");

    const bridge = await upgrades.deployProxy(contractFactory1, [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress, signer.address]);

    await bridge.waitForDeployment();

    const bridgeAddress1 = await bridge.getAddress();
    console.log(
        `Bridge contract deployed at ${bridgeAddress1}`
    );

    return

    const bridgeAddress = "0x5eb3Bc0a489C5A8288765d2336659EbCA68FCd00";

    const contractFactory = await ethers.getContractFactory("Bridge");

    const bridgeContract = (await ethers.getContractFactory("Bridge")).attach(bridgeAddress);

    const prepare = await upgrades.prepareUpgrade(bridgeAddress, contractFactory);

    console.log("prepare",prepare.toString());
    
    const hash = keccak256(
        encoder.encode(["address"], [prepare.toString()])
    );

    console.log(hash);

    const hexifiedDataHash = hexStringToByteArray(hash);

    const signature = await signer.signMessage(hexifiedDataHash);

    console.log(await bridgeContract.validators(signer.address));

    // await bridgeContract.upgrade(prepare.toString(), [[signer.address, signature]])

    // console.log("unique implementation code", await bridgeContract.uniqueImplementations(prepare.toString()));

    const upgraded = await upgrades.upgradeProxy(bridgeAddress, contractFactory);

    console.log("upgraded", await upgraded.getAddress());

    const contract = (await ethers.getContractFactory("Bridge")).attach(bridgeAddress);

    console.log(await contract.getData());
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
