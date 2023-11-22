import { Address, beginCell, toNano } from 'ton-core';
import { Bridge } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton-community/blueprint';
import { testKey } from '@tact-lang/emulator';
import { Parent } from '../build/Bridge/tact_Parent';
import { Todo } from '../build/Bridge/tact_Todo';
import { A } from '../build/Bridge/tact_A';
import { Gando } from '../build/Bridge/tact_Gando';

export async function run(provider: NetworkProvider) {

    let key = testKey('wallet-key');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));
    let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);

    const bridge = provider.open(await Gando.fromInit());

    // await bridge.send(
    //     provider.sender(),
    //     {
    //         value: toNano('0.05'),
    //     },
    //     'Deploy',
    // );

    // await provider.waitForDeploy(bridge.address);

    await bridge.send(
        provider.sender(),
        {
            value: toNano('0.05'),
        },
        {
            $$type: "Gnado", address: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, chain: "TON"
        },

    );

    await provider.waitForDeploy(bridge.address);

}
