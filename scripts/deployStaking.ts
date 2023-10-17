

import { ethers } from "hardhat";

async function main() {


    const erc20TokenName = "DummyERC20"
    const erc20TokenSymbol = "DUM"
    const InitialSupply = "10000"
    const stakingAmount_ = "1"

    const dummyERC20 = await ethers.deployContract("DummyERC20", [erc20TokenName, erc20TokenSymbol, ethers.parseEther(InitialSupply)]);
    await dummyERC20.waitForDeployment();
    
    console.log('DummyERC20 deployed to:', dummyERC20.target, await dummyERC20.symbol());


    const stakingAmount = ethers.parseEther(stakingAmount_);

    const stakingContract = await ethers.deployContract("XpStaking", [stakingAmount, dummyERC20.target]);

    await stakingContract.waitForDeployment();

    console.log(
        `Staking contract deployed at ${stakingContract.target}`
    );
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
