import { ethers } from "hardhat";

async function main() {
    
    const erc20TokenAddress = '0x8cf8238abf7b933Bf8BB5Ea2C7E4Be101c11de2A';
    const stakingAmount_ = '1';

    const stakingAmount = ethers.parseEther(stakingAmount_);

    const stakingContract = await ethers.deployContract("ERC20Staking", [stakingAmount, erc20TokenAddress]);

    await stakingContract.waitForDeployment();

    console.log(
        `Staking contract deployed at ${stakingContract.target}`
    );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
