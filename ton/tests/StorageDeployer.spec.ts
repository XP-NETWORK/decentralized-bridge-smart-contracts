import { Bridge } from "../wrappers/Bridge";
import { ContractSystem, testKey } from "@tact-lang/emulator";
import { Address, beginCell, toNano } from "ton-core";
import { sign } from "ton-crypto";
import { NFTStorageDeployer } from "../build/Bridge/tact_NFTStorageDeployer";
import { NFTCollectionDeployer } from "../build/Bridge/tact_NFTCollectionDeployer";

describe('wallet', () => {
    it('should deploy', async () => {

        // Create wallet
        let key = testKey('wallet-key');
        let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
        let system = await ContractSystem.create();
        let treasure = system.treasure('treasure');
        let contract = system.open(await Bridge.fromInit(publicKey, "TON"));
        let tracker = system.track(contract.address);
        await contract.send(treasure, { value: toNano('10') }, 'Deploy');
        await system.run();

        // Create executor
        // expect(await contract.getPublicKey()).toBe(publicKey);
        // expect(await contract.getWalletId()).toBe(0n);
        // expect(await contract.getSeqno()).toBe(0n);
        console.log(toNano('10').toString());
        console.log(contract.address)
        console.log(await contract.getCollectionDeployer());
        console.log(await contract.getStorageDeployer());

        await contract.send(treasure, { value: toNano('10') }, 'Lock721');


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
        await system.run();
        expect(tracker.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(1n);

        // // Send empty message
        // await contract.send(treasure, { value: toNano(1) }, 'notify');
        // await system.run();
        // expect(tracker.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(2n);

        // // Send comment message
        // await contract.send(treasure, { value: toNano(1) }, null);
        // await system.run();
        // expect(tracker.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(3n);
    });
});