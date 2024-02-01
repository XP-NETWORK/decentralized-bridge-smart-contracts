

import { ethers } from "hardhat";

async function main() {

    const CHAIN = "HEDERA";
    const FEE = "1000000";
    const RR = "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"
    const BRIDGE_STORAGE = "0xaeD15b9AEd9401658A177abaab5854f31973F5C8"


    const bsf = await ethers.getContractFactory("BridgeStorage")
    const bsfc = bsf.attach(BRIDGE_STORAGE);
    //@ts-ignore
    const addf = await bsfc.changeChainFee(CHAIN, FEE, {
        gasPrice: 5000000,
    })
    console.log(`Changed chain fee for ${CHAIN} -> ${FEE} at hash:`, addf.hash)
    // @ts-ignore
    const addrr = await bsfc.changeChainRoyaltyReceiver(CHAIN, RR, {
        gasPrice: 5000000,
    })
    console.log(`Changed royalty receiver for ${CHAIN} -> ${RR} at hash:`, addf.hash)

}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
