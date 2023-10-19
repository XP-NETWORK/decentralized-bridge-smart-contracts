

import { ethers } from "hardhat";

async function main() {

    const bootstrapValidator = "0xyouraddress"
    const chainFees  = [
        ["BSC", "100000000000000"],
        ["ETH", "100000000000000"],
    ] 
    const bridgeStorage = await ethers.deployContract("BridgeStorage", [bootstrapValidator, chainFees]);

    await bridgeStorage.waitForDeployment();

    console.log(
        `BridgeStorage contract deployed at ${bridgeStorage.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
