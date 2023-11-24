

import { ethers } from "hardhat";

async function main() {

    const stakingAmount_ = "100"

    const erc20TokenName = "ERC20Token"
    const erc20TokenSymbol = "ERC"
    const InitialSupply = "10000000"

    const dummyERC20 = await ethers.deployContract("ERC20Token", [erc20TokenName, erc20TokenSymbol, ethers.parseEther(InitialSupply)]);
    await dummyERC20.waitForDeployment();

    console.log('DummyERC20 deployed to:', dummyERC20.target, await dummyERC20.symbol());

    const transferTx =await dummyERC20.transfer("0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23", "10000000")
    await transferTx.wait()
    const stakingAmount = ethers.parseEther(stakingAmount_);

    const stakingContract = await ethers.deployContract("ERC20Staking", [stakingAmount, dummyERC20.target]);

    await stakingContract.waitForDeployment();

    console.log(
        `Staking contract deployed at ${stakingContract.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
