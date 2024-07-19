import { Address } from '@ton/core';
import { NetworkProvider } from '@ton/blueprint';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { toNano } from '@ton/core';

export async function run(provider: NetworkProvider) {

    const nftCollection = provider.open(NftCollection.fromAddress(Address.parseFriendly("EQCRS4p3hdTdwzAGojWBNRuNOzuVYGtBlxur8qiw-TptzspW").address));

    await nftCollection.send(
        provider.sender(),
        {
            value: toNano("0.05"),
        },
        "Mint"
    )

    await provider.waitForDeploy(nftCollection.address);

}
