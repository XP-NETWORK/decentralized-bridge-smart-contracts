import { Bridge, SignerAndSignature, NewValidator, storeNewValidator, loadLock721, storeLock721 } from "../wrappers/Bridge";
import { ContractSystem, testKey } from "@tact-lang/emulator";
import { TonClient } from "ton";
import { Address, Dictionary, Slice, beginCell, toNano } from "ton-core";
import { sign } from "ton-crypto";

describe('wallet', () => {
    it('should deploy', async () => {


        let adfs = beginCell().storeStringTail("b5ee9c720101070100ac00048897149c73000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010102030400144d554c54495645525358007c657264316d3232396b78383574396a73616d6a7578707536736a7475366a77733771346c65736e65396d35676465783967387073366e397363776b32763000086c756c6102000506000a67616e646f0006544f4e").endCell().beginParse();


        console.log(loadLock721(adfs));

        const client = new TonClient({
            endpoint: "https://testnet.tonapi.io"
        });








        // // let key = testKey('wallet-key');
        // // console.log("public", key.publicKey.toString("hex"));
        // // console.log("secret", key.secretKey.toString("hex"));
        // // Create wallet

        // let initkey = testKey('wallet-key');
        // let newkey = testKey('wallet-keys');


        // let publicKey = beginCell().storeBuffer(initkey.publicKey).endCell().beginParse().loadUintBig(256);
        // let newpublicKey = beginCell().storeBuffer(newkey.publicKey).endCell().beginParse().loadUintBig(256);

        // let system = await ContractSystem.create();
        // let treasure = system.treasure('treasure');
        // let contract = system.open(await Bridge.fromInit(publicKey));
        // let tracker = system.track(contract.address);
        // await contract.send(treasure, { value: toNano('10') }, 'Deploy');
        // await system.run();


        // let newValidator: NewValidatorKey = {
        //     $$type: "NewValidatorKey",
        //     key: newpublicKey
        // }

        // let newValidator1: NewValidatorKey = {
        //     $$type: "NewValidatorKey",
        //     key: publicKey
        // }

        // let signature = sign(beginCell().store(storeNewValidatorKey(newValidator)).endCell().hash(), initkey.secretKey);

        // let sig: Sig = {
        //     $$type: 'Sig',
        //     key: publicKey,
        //     signature: beginCell().storeBuffer(signature).endCell()
        // };

        // let dictA = Dictionary.empty<bigint, Sig>().set(0n, sig);

        // console.log("sig", signature.toString("hex"));
        // console.log("public key", initkey.publicKey.toString("hex"));
        // console.log("message", newkey.publicKey.toString("hex"));


        // await contract.send(
        //     treasure,
        //     {
        //         value: toNano('0.05'),
        //     },
        //     {
        //         $$type: 'AddValidator',
        //         newValidatorPublicKey: newValidator,
        //         sigs: dictA,
        //         len: beginCell().storeUint(1, 256).endCell().beginParse().loadUintBig(256),
        //     }
        // );



        // Send transfer and check seqno
        // let transfer: Transfer = {
        //     $$type: 'Transfer',
        //     seqno: 0n,
        //     mode: 1n,
        //     amount: toNano(10),
        //     to: treasure.address,
        //     body: null
        // };
        // let transfer1: Transfer = {
        //     $$type: 'Transfer',
        //     seqno: 0n,
        //     mode: 1n,
        //     amount: toNano(10),
        //     to: treasure.address,
        //     body: beginCell().storeBuffer(Buffer.from("Gando")).endCell()
        // };
        // let signature = sign(beginCell().store(storeTransfer(transfer1)).endCell().hash(), key.secretKey);
        // await contract.send(treasure, { value: toNano(1) }, {
        //     $$type: 'TransferMessage',
        //     transfer: transfer1,
        //     signature: beginCell().storeBuffer(signature).endCell(),
        //     key: publicKey
        // });
        // await system.run();

        // expect(tracker.collect()).toMatchSnapshot();
    });
});