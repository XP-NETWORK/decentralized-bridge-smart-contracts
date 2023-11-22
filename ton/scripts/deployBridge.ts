import { Address, Dictionary, beginCell, toNano } from 'ton-core';
import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton-community/blueprint';
import { testKey } from '@tact-lang/emulator';
import { NFTCollectionDeployer } from '../build/Bridge/tact_NFTCollectionDeployer';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { sign } from 'ton-crypto';
import { NftItem } from '../build/Bridge/tact_NftItem';

export async function run(provider: NetworkProvider) {

    let key = testKey('dfdsfdssss');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));

    let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);

    const bridge = provider.open(await Bridge.fromInit(publicKey, Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, "TON"));

    // const nftItem = provider.open(await NftItem.fromAddress(Address.parseFriendly("EQCAGWMiXRrQONFiv5_RyTBVjmL--ILd4SwyP6cRWItW5_GV").address));

    // console.log(await nftItem.getGetNftData());
    

    // return
    // const bridge = provider.open(await Bridge.fromInit());

    // await bridge.send(
    //     provider.sender(),
    //     {
    //         value: toNano('0.3'),
    //     },
    //     'Deploy',
    // );

    // await provider.waitForDeploy(bridge.address);


    // return

    const OFFCHAIN_CONTENT_PREFIX = 0x01;

    const string_first = "https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/"; // Change to the content URL you prepared

    let newContent = beginCell().storeInt(0x01, 8).storeStringRefTail("https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/").endCell();

    // The Transaction body we want to pass to the smart contract
    let body = beginCell().storeUint(0, 32).storeStringTail("Mint").endCell();


    let claimData: ClaimData = {
        $$type: "ClaimData",
        data1: {
            $$type: 'ClaimData1',
            tokenId: 64n,
            sourceChain: 'BSC',
            destinationChain: 'TON',
            destinationUserAddress: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
            tokenAmount: 1n
        },
        data2: {
            $$type: 'ClaimData2',
            name: 'Gando',
            symbol: 'Gando',
            nftType: 'singular',
        },
        data3: {
            $$type: 'ClaimData3',
            sourceNftContractAddress: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386',
            fee: toNano("0.1"),
            royaltyReceiver: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
            metadata: 'https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/0.json',
        },
        data4: {
            $$type: 'ClaimData4',
            newContent: newContent,
            transactionHash: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908fs386',
            royalty: {
                $$type: 'RoyaltyParams',
                numerator: 1000n,
                denominator: 350n,
                destination: bridge.address
            }
        }
    }

    let signature = sign(beginCell().store(storeClaimData(claimData)).endCell().hash(), key.secretKey);

    let sig: SignerAndSignature = {
        $$type: 'SignerAndSignature',
        key: publicKey,
        signature: beginCell().storeBuffer(signature).endCell()
    };

    let dictA = Dictionary.empty<bigint, SignerAndSignature>().set(0n, sig);



    console.log(storeClaimData(claimData).toString());

    await bridge.send(
        provider.sender(),
        {
            value: toNano('2.8')
        }, {
        $$type: "ClaimNFT721",
        data: claimData,
        len: 1n,
        signatures: dictA
    });


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

    await provider.waitForDeploy(bridge.address);


    // console.log(await bridge.getCollectionDeployer());

    // console.log(await bridge.getStorageDeployer());

}
