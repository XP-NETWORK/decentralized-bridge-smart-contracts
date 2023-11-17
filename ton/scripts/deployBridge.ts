import { Address, beginCell, toNano } from 'ton-core';
import { Bridge } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton-community/blueprint';
import { testKey } from '@tact-lang/emulator';
import { NFTCollectionDeployer } from '../build/Bridge/tact_NFTCollectionDeployer';
import { NftCollection } from '../build/Bridge/tact_NftCollection';

export async function run(provider: NetworkProvider) {

    let key = testKey('wallet-ncccxxxertdddyuaamm');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));
    let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);

    const bridge = provider.open(await Bridge.fromInit(publicKey, "TON"));
    const CollectionDeployer = provider.open(await NFTCollectionDeployer.fromInit(bridge.address));

    await bridge.send(
        provider.sender(),
        {
            value: toNano('0.3'),
        },
        'Deploy',
    );

    await provider.waitForDeploy(bridge.address);



    const OFFCHAIN_CONTENT_PREFIX = 0x01;

    const string_first = "https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/"; // Change to the content URL you prepared

    let newContent = beginCell().storeInt(OFFCHAIN_CONTENT_PREFIX, 8).storeStringRefTail(string_first).endCell();

    // The Transaction body we want to pass to the smart contract
    let body = beginCell().storeUint(0, 32).storeStringTail("Mint").endCell();


    // const Collection = provider.open(await NftCollection.fromInit(Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, newContent, {
    //     $$type: "RoyaltyParams", denominator: 1000n,
    //     destination: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
    //     numerator: 350n
    // }));


    // await Collection.send(
    //     provider.sender(), {
    //     value: toNano("0.3"),
    // }, "CollectionDeploy");

    // // ===== Parameters =====
    // // Replace owner with your address
    await bridge.send(
        provider.sender(),
        {
            value: toNano('0.3'),
        },
        {
            $$type: 'Gando', content: newContent, royalty: {
                $$type: "RoyaltyParams",
                denominator: 1000n,
                destination: bridge.address,
                numerator: 350n
            }
        }
    );

    await provider.waitForDeploy(bridge.address);


    // console.log(await bridge.getCollectionDeployer());

    // console.log(await bridge.getStorageDeployer());

    // console.log(await bridge.getCollections(1n));

}
