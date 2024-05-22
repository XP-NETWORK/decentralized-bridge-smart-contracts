import { Address, beginCell, toNano } from '@ton/core';
import { TestnetNftCollection } from '../wrappers/ExampleNft';
import { NetworkProvider } from '@ton/blueprint';

export async function run(provider: NetworkProvider) {
    const exampleNft = provider.open(
        await TestnetNftCollection.fromInit(
            Address.parse('EQDAnW6XFiIVI_s0ZD2rXZuyM-GynJFfJQOPXwz_3rCIA9db'),
            beginCell().endCell(),
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
            value: toNano('0.05'),
        },
        {
            $$type: "Deploy",
            queryId: 213n
        }
    );

    await provider.waitForDeploy(exampleNft.address);

    // run methods on `exampleNft`
}
