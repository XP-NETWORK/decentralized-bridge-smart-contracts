

import { ethers } from "hardhat";

async function main() {

    const CHAIN = "TON";
    // const FEE = "1000000";
    const RR = "UQBPHMmq9U8X-S3YmsPKpKIBvO4ulsdONM9fLw_-WoZAux9_"
    const BRIDGE_STORAGE = "0x6e372D7fe53F4B7Baa3543Deffe6B87833846D37"


    const bsf = await ethers.getContractFactory("BridgeStorage")
    const bsfc = bsf.attach(BRIDGE_STORAGE);
    //@ts-ignore
    // const addf = await bsfc.changeChainFee(CHAIN, FEE, {
    //     gasPrice: 5000000,
    // })
    // console.log(`Changed chain fee for ${CHAIN} -> ${FEE} at hash:`, addf.hash)
    // @ts-ignore
    const addrr = await bsfc.changeChainRoyaltyReceiver(CHAIN, RR, {
        gasPrice: 5000000,
    })
    console.log(`Changed royalty receiver for ${CHAIN} -> ${RR} at hash:`, addrr.hash)

}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
