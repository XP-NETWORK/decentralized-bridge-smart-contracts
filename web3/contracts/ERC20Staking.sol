// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Uncomment the following line if you want to use the hardhat console.log for debugging purposes.
// import "hardhat/console.sol";

struct ValidatorAddressAndChainType {
    string validatorAddress;
    string chainType;
}

/**
 * @title ERC20Staking
 * @dev A contract for staking a specific amount of XP tokens.
 */
contract ERC20Staking {
    // The required amount for staking.
    uint256 public stakingAmount;

    // The token being staked.
    IERC20 public ERC20Token;

    // Mapping to track the staking balances of individual addresses.
    mapping(address => uint256) public stakingBalances;

    // Event emitted when a user stakes tokens.
    event Staked(address sender, uint256 amount, ValidatorAddressAndChainType[] validatorAddressAndChainType);

    /**
     * @dev Contract constructor that initializes the staking amount and token contract.
     * @param _stakingAmount Amount that is required to stake.
     * @param _ERC20Token Address of the ERC20 token to be staked.
     */
    constructor(uint256 _stakingAmount, address _ERC20Token) {
        stakingAmount = _stakingAmount;
        ERC20Token = IERC20(_ERC20Token);
    }

    /**
     * @dev Allows users to stake a specific amount of XP tokens.
     */
    function stakeERC20(ValidatorAddressAndChainType[] memory _validatorAddressAndChainType) public {
        require(stakingBalances[msg.sender] == 0, "You can only stake once");

        // @TODO chains should be unique

        // Transfer the staking amount from the staker to this contract.
        ERC20Token.transferFrom(msg.sender, address(this), stakingAmount);

        // Update the staking balance for the staker.
        stakingBalances[msg.sender] += stakingAmount;

        // Emit a staking event.
        emit Staked(msg.sender, stakingAmount, _validatorAddressAndChainType);
    }

    /**
     * @dev Add new chains.
     */
    function addNewChains(ValidatorAddressAndChainType[] memory _validatorAddressAndChainType) public {
        require(stakingBalances[msg.sender] > 0, "You have to stake once");

        // @TODO chains should be unique
        // Emit a staking event.
        emit Staked(msg.sender, stakingAmount, _validatorAddressAndChainType);
    }
}
