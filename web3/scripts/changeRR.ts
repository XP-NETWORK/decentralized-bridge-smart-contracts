// 5000000000

import { ethers } from "hardhat";
import { BridgeStorage__factory } from "../contractsTypes/factories/contracts/BridgeStorage__factory";
//  contractsTypes/factories/contracts/BridgeStorage__factory.ts
async function main() {

    const BRIDGE_STORAGE = "0x04cAEd1763B2C121D92FcaEaB41BFfe3EAB57EFC"

    const bsf = await ethers.getContractFactory("BridgeStorage")
    const bsfc = bsf.attach(BRIDGE_STORAGE);
    // @ts-ignore

    console.log(
        // await bsfc.changeChainFee("TEZOS","1000000"),
        await bsfc.chainFee("TEZOS")
    );

    // await bsfc.changeChainRoyaltyReceiver("TEZOS",RR);

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
