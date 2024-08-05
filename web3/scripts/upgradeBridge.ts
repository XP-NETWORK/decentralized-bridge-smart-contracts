import { ContractFactory, keccak256 } from "ethers";
import { ethers, upgrades } from "hardhat";
import { encoder, hexStringToByteArray } from "../test/utils";
import { UpgradeProxyOptions } from "@openzeppelin/hardhat-upgrades/dist/utils";


async function main() {
    const [signer] = await ethers.getSigners();

    // const collectionDeployerContract = await ethers.deployContract("NFTCollectionDeployer");
    // await collectionDeployerContract.waitForDeployment();
    // const collectionDeployerContractAddress = collectionDeployerContract.target
    // console.log(
    //     `NFTCollectionDeployer contract deployed at ${collectionDeployerContractAddress}`
    // );


    // const storageDeployerContract = await ethers.deployContract("NFTStorageDeployer");
    // await storageDeployerContract.waitForDeployment();
    // const storageDeployerContractAddress = storageDeployerContract.target
    // console.log(
    //     `StorageDeployerContract contract deployed at ${storageDeployerContractAddress}`
    // );

    // const bootstrapValidator = [signer.address]
    // const chainSymbol = "ETH"
    // const contractFactory1 = await ethers.getContractFactory("Bridge");

    // const bridge = await upgrades.deployProxy(contractFactory1, [bootstrapValidator, chainSymbol, collectionDeployerContractAddress, storageDeployerContractAddress, signer.address]);

    // await bridge.waitForDeployment();

    // const bridgeAddress1 = await bridge.getAddress();
    // console.log(
    //     `Bridge contract deployed at ${bridgeAddress1}`
    // );

    // return

    const bridgeAddress = "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9";

    const contractFactory = await ethers.getContractFactory("Bridge");

    const bridgeContract = (await ethers.getContractFactory("Bridge")).attach(bridgeAddress);

    const hash = keccak256(
        encoder.encode(["bytes"], [contractFactory.bytecode])
    );
    console.log(hash);

    const hexifiedDataHash = hexStringToByteArray(hash);

    const signature = await signer.signMessage(hexifiedDataHash)

    console.log(await bridgeContract.validators(signer.address));

    await bridgeContract.upgrade(contractFactory.bytecode, [[signer.address, signature]])

    console.log("unique byte code", await bridgeContract.uniqueByteCode(hexifiedDataHash));

    console.log("upgradeable", await bridgeContract.upgradeable(hexifiedDataHash));


    const upgraded = await upgrades.upgradeProxy(bridgeAddress, contractFactory);

    console.log("upgraded", await upgraded.getAddress());

    const contract = (await ethers.getContractFactory("Bridge")).attach(bridgeAddress);

    console.log(await contract.getData());
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});

function encodeCall(factory: ContractFactory, call: UpgradeProxyOptions['call']): string | undefined {
    if (!call) {
        return undefined;
    }

    if (typeof call === 'string') {
        call = { fn: call };
    }

    return factory.interface.encodeFunctionData(call.fn, call.args ?? []);
}
