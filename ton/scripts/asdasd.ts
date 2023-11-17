import { Address, Slice, beginCell } from "ton-core";

(async () => {
    let adsf = Buffer.from("TON");
    console.log(adsf);
    console.log(adsf.toString());
    console.log(beginCell().storeBuffer(adsf).endCell());
    console.log(beginCell().storeBuffer(adsf).endCell().beginParse().loadStringTail());


    const a = beginCell().storeBuffer(Buffer.from("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh")).storeBuffer(Buffer.from("TON")).endCell().hash();

    console.log(beginCell().storeBuffer(a).endCell().beginParse().loadUintBig(32));

})();