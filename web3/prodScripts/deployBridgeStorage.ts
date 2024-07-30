

import { ethers } from "hardhat";

async function main() {

    const bootstrapValidator = "0xe9dFDea6Da1Dc1906027f5412Dc137E7D9E2A9e4"
    const chainFees  = [
        ["BSC", "9829", "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"],
        ["ETH", "1629", "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"],
        ["MATIC", "1062", "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"],
        ["HEDERA", "7568", "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"],
    ] 
    const bridgeStorage = await ethers.deployContract("BridgeStorage", [bootstrapValidator, chainFees]);

    await bridgeStorage.waitForDeployment();

    console.log(
        `Storage contract deployed at ${bridgeStorage.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
