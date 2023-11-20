import { Address, Dictionary, beginCell, toNano } from 'ton-core';
import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton-community/blueprint';
import { testKey } from '@tact-lang/emulator';
import { NFTCollectionDeployer } from '../build/Bridge/tact_NFTCollectionDeployer';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { sign } from 'ton-crypto';

export async function run(provider: NetworkProvider) {

    let key = testKey('firstValidator');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));

    let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);

    const bridge = provider.open(await Bridge.fromInit(publicKey, Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, "TON"));

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


    // let claimData: ClaimData = {
    //     $$type: "ClaimData",
    //     destinationChain: "TON",
    //     destinationUserAddress: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
    //     fee: 0n,
    //     metadata: "https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/0.json",
    //     name: "Gando",
    //     nftType: "singular",
    //     royalty: {
    //         $$type: "RoyaltyParams",
    //         denominator: 1000n,
    //         destination: bridge.address,
    //         numerator: 350n
    //     },
    //     royaltyReceiver: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
    //     sourceChain: "BSC",
    //     sourceNftContractAddress: "0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386",
    //     symbol: "Gando",
    //     tokenAmount: 1n,
    //     tokenId: 64n,
    //     transactionHash: "0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386"
    // }

    // let signature = sign(beginCell().store(storeClaimData(claimData)).endCell().hash(), key.secretKey);

    // let sig: SignerAndSignature = {
    //     $$type: 'SignerAndSignature',
    //     key: publicKey,
    //     signature: beginCell().storeBuffer(signature).endCell()
    // };

    // let dictA = Dictionary.empty<bigint, SignerAndSignature>().set(0n, sig);



    // console.log(storeClaimData(claimData).toString());
    
    // await bridge.send(
    //     provider.sender(),
    //     {
    //         value: toNano('0.3')
    //     }, {
    //     $$type: "ClaimNFT721",
    //     data: claimData,
    //     len: 1n,
    //     signatures: dictA
    // });


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
    // await bridge.send(
    //     provider.sender(),
    //     {
    //         value: toNano('0.3'),
    //     },
    //     {
    //         $$type: 'Gando', content: newContent, royalty: {
    //             $$type: "RoyaltyParams",
    //             denominator: 1000n,
    //             destination: bridge.address,
    //             numerator: 350n
    //         }
    //     }
    // );

    // await provider.waitForDeploy(bridge.address);


    console.log(await bridge.getCollectionDeployer());

    console.log(await bridge.getStorageDeployer());

}
