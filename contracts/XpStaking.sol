// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";


contract XpStaking {
    uint256 public stakingAmount;
    IERC20 public xpToken;
    mapping(address => uint256) public stakingBalances;


    event Staked(address indexed user, uint256 amount);


    constructor(uint256 _stakingAmount, address _erc20Token) {
        stakingAmount = _stakingAmount;
        xpToken = IERC20(_erc20Token);
    }

    function stakeXP(uint256 amount) public {
        require(amount == stakingAmount, "Staking amount must be equal to the predefined staking amount.");
        
        xpToken.transferFrom(msg.sender, address(this), stakingAmount);

        stakingBalances[msg.sender] += stakingAmount;
        emit Staked(msg.sender, stakingAmount);
    }
}