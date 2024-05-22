import { Blockchain, SandboxContract, TreasuryContract } from '@ton/sandbox';
import { toNano } from '@ton/core';
import { ExampleNft } from '../wrappers/ExampleNft';
import '@ton/test-utils';

describe('ExampleNft', () => {
    let blockchain: Blockchain;
    let deployer: SandboxContract<TreasuryContract>;
    let exampleNft: SandboxContract<ExampleNft>;

    beforeEach(async () => {
        blockchain = await Blockchain.create();

        exampleNft = blockchain.openContract(await ExampleNft.fromInit());

        deployer = await blockchain.treasury('deployer');

        const deployResult = await exampleNft.send(
            deployer.getSender(),
            {
                value: toNano('0.05'),
            },
            {
                $$type: 'Deploy',
                queryId: 0n,
            }
        );

        expect(deployResult.transactions).toHaveTransaction({
            from: deployer.address,
            to: exampleNft.address,
            deploy: true,
            success: true,
        });
    });

    it('should deploy', async () => {
        // the check is done inside beforeEach
        // blockchain and exampleNft are ready to use
    });
});
