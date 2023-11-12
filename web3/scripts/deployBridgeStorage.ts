

import { ethers } from "hardhat";

async function main() {

    const bootstrapValidator = "0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"
    const chainFees  = [
        ["BSC", "100000000000000", "0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"],
        ["ETH", "100000000000000", "0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"],
        ["MATIC", "100000000000000", "0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"],
        ["MULTIVERSX", "100000000000000", "2130d2c16f919f634de847801cdccefbbc1f89bdd2524d5b6b94edbf821b2b00"],
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