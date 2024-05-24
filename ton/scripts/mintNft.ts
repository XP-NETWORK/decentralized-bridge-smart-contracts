
import { Address, beginCell, toNano } from '@ton/core';
import { TestnetNftCollection } from '../wrappers/ExampleNft';
import { NetworkProvider } from '@ton/blueprint';

export async function run(provider: NetworkProvider) {
    const exampleNft = provider.open(
        TestnetNftCollection.fromAddress(Address.parse('kQAZy7l-KkX83h10EbfNSjwClddNrpj0sdziC1JNezo9wSNZ')),
    );

    await exampleNft.send(
        provider.sender(),
        {
            value: toNano('0.1'),
        },
        {
            $$type: 'Mint',
            content: beginCell().storeInt(1, 8).storeStringTail('https://meta.polkamon.com/meta?id=10001852306').endCell(),
            owner: Address.parse('EQBPHMmq9U8X-S3YmsPKpKIBvO4ulsdONM9fLw_-WoZAu0K6'),
            token_id: BigInt(2),
        },
    );

}
