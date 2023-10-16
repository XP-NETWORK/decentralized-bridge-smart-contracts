import { ethers } from "hardhat";

async function main() {

    const erc20TokenAddress = '0xYourERC20TokenAddressHere';
    const stakingAmount_ = '100';
    const stakingAmount = ethers.parseEther(stakingAmount_);

    const stakingContract = await ethers.deployContract("XpStaking", [stakingAmount, erc20TokenAddress]);

    await stakingContract.waitForDeployment();

    console.log(
        `Staking contract deployed at ${stakingContract}`
    );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
