// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Uncomment the following line if you want to use the hardhat console.log for debugging purposes.
// import "hardhat/console.sol";

/**
 * @title XpStaking
 * @dev A contract for staking a specific amount of XP tokens.
 */
contract XpStaking {
    
    // The required amount for staking.
    uint256 public stakingAmount;
    
    // The token being staked.
    IERC20 public xpToken;
    
    // Mapping to track the staking balances of individual addresses.
    mapping(address => uint256) public stakingBalances;

    // Event emitted when a user stakes tokens.
    event Staked(address indexed user, uint256 amount);

    /**
     * @dev Contract constructor that initializes the staking amount and token contract.
     * @param _stakingAmount Amount that is required to stake.
     * @param _erc20Token Address of the ERC20 token to be staked.
     */
    constructor(uint256 _stakingAmount, address _erc20Token) {
        stakingAmount = _stakingAmount;
        xpToken = IERC20(_erc20Token);
    }

    /**
     * @dev Allows users to stake a specific amount of XP tokens.
     * @param amount The amount of tokens to be staked.
     */
    function stakeXP(uint256 amount) public {
        require(amount == stakingAmount, "Staking amount must be equal to the predefined staking amount.");

        // Transfer the staking amount from the staker to this contract.
        xpToken.transferFrom(msg.sender, address(this), stakingAmount);
        
        // Update the staking balance for the staker.
        stakingBalances[msg.sender] += stakingAmount;

        // Emit a staking event.
        emit Staked(msg.sender, stakingAmount);
    }
}
