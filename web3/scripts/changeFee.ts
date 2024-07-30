// 5000000000

import { ethers } from "hardhat";
import { BridgeStorage__factory } from "../contractsTypes/factories/contracts/BridgeStorage__factory";
//  contractsTypes/factories/contracts/BridgeStorage__factory.ts
async function main() {

    const CHAIN = "HEDERA";
    const FEE = "5000000000";
    const RR = "0xc343bB8e508F5330F3bA503bD2aF82bcF968bc40"
    const BRIDGE_STORAGE = "0x463E4A5eA947D9DFC250687E469AA487cbD8e1A7"

    const bsf = await ethers.getContractFactory("BridgeStorage")
    const bsfc = bsf.attach(BRIDGE_STORAGE);
    // @ts-ignore
    const addf = await bsfc.changeChainFee(CHAIN, FEE)
    console.log(`Changed chain fee for ${CHAIN} -> ${FEE} at hash:`, addf.hash)

    // @ts-ignore
    // const addrr = await bsfc.changeChainRoyaltyReceiver(CHAIN, RR, {
    //     gasPrice: 5000000,
    // })
    // console.log(`Changed royalty receiver for ${CHAIN} -> ${RR} at hash:`, addf.hash)

}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
