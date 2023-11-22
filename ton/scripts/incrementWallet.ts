import { Address, beginCell, toNano } from 'ton-core';
import { Bridge, Transfer, storeTransfer } from '../wrappers/Bridge';
import { NetworkProvider, sleep } from '@ton-community/blueprint';
import { sign } from 'ton-crypto';

export async function run(provider: NetworkProvider, args: string[]) {
    const ui = provider.ui();

    const address = Address.parse(args.length > 0 ? args[0] : await ui.input('Bridge address'));

    if (!(await provider.isContractDeployed(address))) {
        ui.write(`Error: Contract at address ${address} is not deployed!`);
        return;
    }

    const bridge = provider.open(Bridge.fromAddress(address));

    let publicKey = Buffer.from("5fd9df09c6326a7a6573681260710aa79c9618327e2251f72c5150db8f177ae1", "hex");
    let publicKeyI = beginCell().storeBuffer(publicKey).endCell().beginParse().loadUintBig(256);
    let secretKey = Buffer.from("6fe9d0c80dd6dc67385fdd50997bd4b90de635073900114eaaf144797bd891b55fd9df09c6326a7a6573681260710aa79c9618327e2251f72c5150db8f177ae1", "hex");


    let transfer: Transfer = {
        $$type: 'Transfer',
        seqno: 0n,
        mode: 1n,
        amount: toNano(10),
        to: Address.parseFriendly("EQATw7jzaDWNjF9xHDLEMHTTn1ORHLmhuH7LWAGV4nms-4Bk").address,
        body: null
    };
    let transfer1: Transfer = {
        $$type: 'Transfer',
        seqno: 0n,
        mode: 1n,
        amount: toNano(10),
        to: Address.parseFriendly("EQATw7jzaDWNjF9xHDLEMHTTn1ORHLmhuH7LWAGV4nms-4Bk").address,
        body: beginCell().storeBuffer(Buffer.from("Gando")).endCell()
    };
    let signature = sign(beginCell().store(storeTransfer(transfer1)).endCell().hash(), secretKey);
    // await contract.send(treasure, { value: toNano(1) }, {
    //     $$type: 'TransferMessage',
    //     transfer: transfer1,
    //     signature: beginCell().storeBuffer(signature).endCell(),
    //     key: publicKey
    // });

    await bridge.send(
        provider.sender(),
        {
            value: toNano('0.05'),
        },
        {
            $$type: 'TransferMessage',
            transfer: transfer1,
            signature: beginCell().storeBuffer(signature).endCell(),
            key: publicKeyI
        }
    );

    await provider.waitForDeploy(bridge.address);
    ui.write('Waiting for counter to increase...');


    ui.clearActionPrompt();
    ui.write('Counter increased successfully!');
}
