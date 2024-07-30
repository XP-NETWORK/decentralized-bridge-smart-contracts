// 5000000000

import { ethers } from "hardhat";
import { BridgeStorage__factory } from "../contractsTypes/factories/contracts/BridgeStorage__factory";
//  contractsTypes/factories/contracts/BridgeStorage__factory.ts
async function main() {

    const BRIDGE_STORAGE = "0x463E4A5eA947D9DFC250687E469AA487cbD8e1A7"

    const bsf = await ethers.getContractFactory("BridgeStorage")
    const bsfc = bsf.attach(BRIDGE_STORAGE);
    // @ts-ignore
    const addf = await bsfc.getLockNftSignatures("0xa231c00ed1382866788dddd30277772e199c5928d1b0d77a4873af29f9d24782", "HEDERA")
    console.log(addf);

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
