import { Address, beginCell, toNano } from '@ton/core';
import { TestnetNftCollection } from '../wrappers/ExampleNft';
import { NetworkProvider } from '@ton/blueprint';

export async function run(provider: NetworkProvider) {
    const exampleNft = provider.open(
        await TestnetNftCollection.fromInit(
            Address.parse('EQDAnW6XFiIVI_s0ZD2rXZuyM-GynJFfJQOPXwz_3rCIA9db'),
            beginCell()
                .storeInt(1, 8)
                .storeStringRefTail('https://api.jsonbin.io/v3/b/664f3240e41b4d34e4f8302c?meta=false')
                .endCell(),
            {
                $$type: 'RoyaltyParams',
                denominator: 10000n,
                numerator: 10n,
                destination: Address.parse('EQDAnW6XFiIVI_s0ZD2rXZuyM-GynJFfJQOPXwz_3rCIA9db'),
            },
        ),
    );

    await exampleNft.send(
        provider.sender(),
        {
            value: toNano('0.06'),
        },
        {
            $$type: "Deploy",
            queryId: 213n
        }
    );

    await provider.waitForDeploy(exampleNft.address);

    // run methods on `exampleNft`
}
