import { ContractSystem, testKey } from "@tact-lang/emulator";
import { Address, beginCell, toNano } from '@ton/core';
import { sign } from "ton-crypto";
import { NFTCollectionDeployer } from "../build/Bridge/tact_NFTCollectionDeployer";

describe('wallet', () => {
    it('should deploy', async () => {

        

        // Create wallet
        let key = testKey('wallet-key');
        let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
        let system = await ContractSystem.create();
        let treasure = system.treasure('treasure');
        let contract = system.open(await NFTCollectionDeployer.fromInit(Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address));
        let tracker = system.track(contract.address);
        await contract.send(treasure, { value: toNano('10') }, 'Deploy');
        await system.run();

        // Create executor
        // expect(await contract.getPublicKey()).toBe(publicKey);
        // expect(await contract.getWalletId()).toBe(0n);
        // expect(await contract.getSeqno()).toBe(0n);

        // Send transfer and check seqno
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