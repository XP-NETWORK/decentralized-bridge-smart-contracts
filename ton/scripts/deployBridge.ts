import { Address, Dictionary, beginCell, toNano, Message } from 'ton-core';
import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton-community/blueprint';
import { testKey } from '@tact-lang/emulator';
import { NFTCollectionDeployer } from '../build/Bridge/tact_NFTCollectionDeployer';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { sign } from 'ton-crypto';
import { NftItem } from '../build/Bridge/tact_NftItem';
import { Slice } from '@ton/core';

export async function run(provider: NetworkProvider) {

    let key = testKey('aaaapopop');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));

    let publicKey = beginCell().storeBuffer(Buffer.from("5aba2a59ffcef4fc7894ac0682dd419b18a54c30b55d0db0e244f15b1a7f87b2", "hex")).endCell().beginParse().loadUintBig(256);

    const bridge = provider.open(await Bridge.fromInit(publicKey, Address.parseFriendly("EQBauipZ/870/HiUrAaC3UGbGKVMMLVdDbDiRPFbGn+HstxO").address, "TON"));

    console.log(bridge.address);

    const nftItem = provider.open(NftItem.fromAddress(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address));

    // await nftItem.send(
    //     provider.sender(),
    //     {
    //         value: toNano("0.08"),
    //     },
    //     {
    //         $$type: "Transfer",
    //         forward_amount: 0n,
    //         response_destination: Address.parseFriendly("EQDI6P9gheuWLh1euThjFE2muUpa9tp2y49TD6Zz5oOF5gWL").address,
    //         forward_payload: beginCell().endCell(),
    //         new_owner: bridge.address,
    //         custom_payload: null,
    //         query_id: 0n,
    //     }
    // );
    // await provider.waitForDeploy(nftItem.address);

    // return;

    DEPLOY: {
        // await bridge.send(
        //     provider.sender(),
        //     {
        //         value: toNano('0.3'),
        //     },
        //     'Deploy',
        // );

        // await provider.waitForDeploy(bridge.address);

        // return

    }

    CLAIM: {
        const OFFCHAIN_CONTENT_PREFIX = 0x01;

        const string_first = "https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/"; // Change to the content URL you prepared

        let newContent = beginCell().storeInt(0x01, 8).storeStringRefTail("https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/").endCell();

        // The Transaction body we want to pass to the smart contract
        let body = beginCell().storeUint(0, 32).storeStringTail("Mint").endCell();


        // let claimData: ClaimData = {
        //     $$type: "ClaimData",
        //     data1: {
        //         $$type: 'ClaimData1',
        //         tokenId: 64n,
        //         sourceChain: 'BSC',
        //         destinationChain: 'TON',
        //         destinationUserAddress: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
        //         tokenAmount: 1n
        //     },
        //     data2: {
        //         $$type: 'ClaimData2',
        //         name: 'Gando',
        //         symbol: 'Gando',
        //         nftType: 'singular',
        //     },
        //     data3: {
        //         $$type: 'ClaimData3',
        //         sourceNftContractAddress: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386',
        //         fee: toNano("0.1"),
        //         royaltyReceiver: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
        //         metadata: 'https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/0.json',
        //     },
        //     data4: {
        //         $$type: 'ClaimData4',
        //         newContent: newContent,
        //         transactionHash: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908fs386',
        //         royalty: {
        //             $$type: 'RoyaltyParams',
        //             numerator: 1000n,
        //             denominator: 350n,
        //             destination: bridge.address
        //         }
        //     }
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
        //         value: toNano('1')
        //     }, {
        //     $$type: "ClaimNFT721",
        //     data: claimData,
        //     len: 1n,
        //     signatures: dictA
        // });

        // await provider.waitForDeploy(bridge.address);

        // return;


        // const nftTransferDetailsObject = {
        //     tokenId: '51',
        //     sourceChain: 'BSC',
        //     destinationChain: 'TON',
        //     destinationUserAddress: 'EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh',
        //     sourceNftContractAddress: '0x491d6f9f14e0cd58d5094333ae172cdd19c87781',
        //     name: 'DUM',
        //     symbol: 'D',
        //     royalty: '0',
        //     royaltyReceiver: 'EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh',
        //     metadata: 'abc.com/1',
        //     transactionHash: '0xede7297a17783b6385c76a8c2ac27ab112eebdf59bee7fed8e8ff13bb7c6ae58',
        //     tokenAmount: '1',
        //     nftType: 'singular',
        //     fee: '100000000'
        // }
        // // sig: '0x5277a3352b84cecf81802afa7887abac6ce4fa85df5a290bf089f649ddd21a2ac72340c63ae73a2af3c8af5252452584e66e9b921f2a6932e04b231a6d1a940c'

        // const {
        //     tokenId,
        //     sourceChain,
        //     destinationChain,
        //     destinationUserAddress,
        //     sourceNftContractAddress,
        //     name,
        //     symbol,
        //     royalty,
        //     royaltyReceiver,
        //     metadata,
        //     transactionHash,
        //     tokenAmount,
        //     nftType,
        //     fee
        // } = nftTransferDetailsObject;

        // const claimData: ClaimData = {
        //     $$type: "ClaimData",
        //     data1: {
        //         $$type: "ClaimData1",
        //         tokenId: BigInt(tokenId),
        //         destinationChain,
        //         destinationUserAddress: Address.parseFriendly(destinationUserAddress).address,
        //         sourceChain,
        //         tokenAmount: BigInt(tokenAmount)
        //     },
        //     data2: {
        //         $$type: "ClaimData2",
        //         name,
        //         nftType,
        //         symbol
        //     },
        //     data3: {
        //         $$type: "ClaimData3",
        //         fee: BigInt(fee),
        //         metadata,
        //         royaltyReceiver: Address.parseFriendly(royaltyReceiver).address,
        //         // sourceNftContractAddress: beginCell().storeSlice(beginCell().storeAddress(Address.parseFriendly(sourceNftContractAddress).address).endCell().asSlice()).endCell()
        //         sourceNftContractAddress: beginCell().storeSlice(beginCell().storeStringTail(sourceNftContractAddress).endCell().asSlice()).endCell()
        //     },
        //     data4: {
        //         $$type: "ClaimData4",
        //         newContent: beginCell().storeInt(0x01, 8).storeStringRefTail(metadata).endCell(),
        //         royalty: {
        //             $$type: "RoyaltyParams",
        //             numerator: BigInt(10000),
        //             denominator: BigInt(royalty),
        //             destination: Address.parseFriendly(royaltyReceiver).address,
        //         },
        //         transactionHash
        //     }
        // }

        // let signature = sign(beginCell().store(storeClaimData(claimData)).endCell().hash(), Buffer.from("068e14dc8440a571306c2294c9a18b31fc2f853ecc087412aff42c16f9d6dd2d5aba2a59ffcef4fc7894ac0682dd419b18a54c30b55d0db0e244f15b1a7f87b2", "hex"));

        // let sig: SignerAndSignature = {
        //     $$type: 'SignerAndSignature',
        //     key: publicKey,
        //     signature: beginCell().storeBuffer(Buffer.from("3e3835c7ed8154e53f06aaed9182e2cdb1d6a9720644fd12377b24ccc1b845ea699d45b4f880b2c067d18571359747214c2724a65e3b885347d295f2a65d860f", "hex")).endCell()
        // };

        // let dictA = Dictionary.empty<bigint, SignerAndSignature>().set(0n, sig);



        // console.log(storeClaimData(claimData).toString());

        // await bridge.send(
        //     provider.sender(),
        //     {
        //         value: toNano('0.8')
        //     }, {
        //     $$type: "ClaimNFT721",
        //     data: claimData,
        //     len: 1n,
        //     signatures: dictA
        // });

        // await provider.waitForDeploy(bridge.address);

        // return;
    }

    LOCK: {
        await bridge.send(
            provider.sender(),
            {
                value: toNano('2')
            },
            {
                $$type: "Lock721",
                destinationChain: "BSC",
                destinationUserAddress: "0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23",
                sourceNftContractAddress: Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address,
                tokenId: 0n
            });

        await provider.waitForDeploy(bridge.address);
    }


    // console.log(await bridge.getCollectionDeployer());

    // console.log(await bridge.getStorageDeployer());

}
