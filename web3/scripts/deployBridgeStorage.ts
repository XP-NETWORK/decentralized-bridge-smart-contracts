

import { ethers } from "hardhat";

async function main() {

    const bootstrapValidator = "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"
    const chainFees  = [
        ["BSC", "100000000000000", "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"],
        ["MATIC", "100000000000000", "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"],
        ["ETH", "100000000000000", "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"],
        ["MULTIVERSX", "100000000000000", "9fb927c978225cb7a93b8b3cd8d8423e176e009dc284c536d9c4372bbe128487"],
        ["TON", "100000000", "EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh"],
        ["SECRET", "10000", "secret196dcgy98spe3v6pwv0j6sf09lc7cv0ucf267j4"],
        ["TEZOS", "10000", "tz1NqGupPkw59EHtc6K1YfMqrmgHAjz4qWoB"],
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
