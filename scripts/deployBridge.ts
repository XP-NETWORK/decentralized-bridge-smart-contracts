import { ethers } from "hardhat";

async function main() {

    const bootstrapValidator = ["0x67081bD856e29d7D7B3028C34Afb331fa6b3186E"]
    const chainSymbol = "BSC"
    const bridge = await ethers.deployContract("Bridge", [bootstrapValidator, chainSymbol]);

    await bridge.waitForDeployment();

    console.log(
        `BridgeStorage contract deployed at ${bridge.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
