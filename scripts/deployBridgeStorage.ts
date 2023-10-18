

import { ethers } from "hardhat";

async function main() {


    const bridgeStorage = await ethers.deployContract("BridgeStorage");

    await bridgeStorage.waitForDeployment();

    console.log(
        `BridgeStorage contract deployed at ${bridgeStorage.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
