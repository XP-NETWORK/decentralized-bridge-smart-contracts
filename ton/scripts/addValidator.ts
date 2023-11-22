import { Address, Builder, Dictionary, DictionaryValue, beginCell, toNano, Sender } from 'ton-core';
import { AddValidator, Bridge, NewValidator, SignerAndSignature, loadSignerAndSignature, storeAddValidator, storeNewValidator, storeSignerAndSignature } from '../wrappers/Bridge';
import { NetworkProvider, sleep } from '@ton-community/blueprint';
import { sign } from 'ton-crypto';
import { testKey } from '@tact-lang/emulator';

export async function run(provider: NetworkProvider, args: string[]) {
    const ui = provider.ui();


    const address = Address.parse(args.length > 0 ? args[0] : await ui.input('Bridge address'));

    if (!(await provider.isContractDeployed(address))) {
        ui.write(`Error: Contract at address ${address} is not deployed!`);
        return;
    }

    const bridge = provider.open(Bridge.fromAddress(address));

    // let publicKey = Buffer.from("5fd9df09c6326a7a6573681260710aa79c9618327e2251f72c5150db8f177ae1", "hex");
    // let publicKeyI = beginCell().storeBuffer(publicKey).endCell().beginParse().loadUintBig(256);
    // let secretKey = Buffer.from("6fe9d0c80dd6dc67385fdd50997bd4b90de635073900114eaaf144797bd891b55fd9df09c6326a7a6573681260710aa79c9618327e2251f72c5150db8f177ae1", "hex");

    // let key = testKey('wallet-keys');
    // console.log("public", key.publicKey.toString("hex"));
    // console.log("secret", key.secretKey.toString("hex"));

    // let newValidatorPublicKeyI = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);

    // let newValidator: NewValidator = {
    //     $$type: "NewValidator",
    //     key: newValidatorPublicKeyI
    // }

    // let newValidator1: NewValidator = {
    //     $$type: "NewValidator",

    //     key: publicKeyI
    // }

    // let signature = sign(beginCell().store(storeNewValidator(newValidator1)).endCell().hash(), secretKey);

    // let sig: SignerAndSignature = {
    //     $$type: 'SignerAndSignature',
    //     key: publicKeyI,
    //     signature: beginCell().storeBuffer(signature).endCell()
    // };

    // let dictA = Dictionary.empty<bigint, SignerAndSignature>().set(0n, sig);

    await bridge.send(
        provider.sender(),
        {
            value: toNano('0.05'),
        },
        {
            $$type: "Lock721",
            destinationChain: "MULTIVERSX",
            destinationUserAddress: "erd1m229kx85t9jsamjuxpu6sjtu6jws7q4lesne9m5gdex9g8ps6n9scwk2v0",
            sourceNftContractAddress: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
            tokenId: 1n

            // newValidatorPublicKey: newValidator,
            // sigs: dictA,
            // len: beginCell().storeUint(dictA.keys.length, 256).endCell().beginParse().loadUintBig(256),
        }
    );

    // bridge.getGetValidator(publicKeyI);
    await provider.waitForDeploy(bridge.address);

    ui.write('Waiting for counter to increase...');
    ui.clearActionPrompt();
    ui.write('Counter increased successfully!');
}


