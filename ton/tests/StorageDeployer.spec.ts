import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from "../wrappers/Bridge";
import { ContractSystem, testKey } from "@tact-lang/emulator";
import { Address, beginCell, toNano } from "ton-core";
import { sign } from "ton-crypto";
import { NFTStorageDeployer } from "../build/Bridge/tact_NFTStorageDeployer";
import { NFTCollectionDeployer } from "../build/Bridge/tact_NFTCollectionDeployer";
import { Gando, Gnado, storeGnado } from "../build/Bridge/tact_Gando";
import { Dictionary } from "ton";
import { NftCollection } from "../build/Bridge/tact_NftCollection";

describe('wallet', () => {
    it('should deploy', async () => {

        // Create wallet
        let key = testKey('firstValidatorss9sss');
        let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
        let system = await ContractSystem.create();
        let treasure = system.treasure('treasure');
        let g: Gnado = {
            $$type: "Gnado", address: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, chain: "TON"
        }
        let contract = system.open(await Bridge.fromInit(publicKey, Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, "TON"));
        let tracker = system.track(contract.address);
        let logger = system.log(contract.address);
        await contract.send(treasure, { value: toNano('10') }, 'Deploy');
        await system.run();

        // Create executor
        // expect(await contract.getPublicKey()).toBe(publicKey);
        // expect(await contract.getWalletId()).toBe(0n);
        // expect(await contract.getSeqno()).toBe(0n);
        // console.log(toNano('10').toString());
        console.log(contract.address);
        console.log(await contract.getCollectionDeployer());
        console.log(await contract.getStorageDeployer());


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
                transactionHash: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386',
                royalty: {
                    $$type: 'RoyaltyParams',
                    numerator: 1000n,
                    denominator: 350n,
                    destination: contract.address
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

        await contract.send(
            treasure,
            {
                value: toNano('2.8')
            }, {
            $$type: "ClaimNFT721",
            data: claimData,
            len: 1n,
            signatures: dictA
        });

        let nftCollection = system.open(await NftCollection.fromInit(contract.address, newContent, {
            $$type: 'RoyaltyParams',
            numerator: 1000n,
            denominator: 350n,
            destination: contract.address
        }));

        let tracker2 = system.track(nftCollection.address);

        // await contract.send(treasure, { value: toNano('10') }, 'Lock721');


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