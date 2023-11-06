// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

/**
 * @dev Structure representing chain to chain fee
 */
struct ChainFee {
    string chain;
    uint256 fee;
    string royaltyReceiver;
}

struct Chain {
    uint256 fee;
    string royaltyReceiver;
}

/**
 * @dev Stucture to store signature with signer public address
 */
struct SignerAndSignature {
    string publicAddress;
    string signature;
}

/**
 * @title BridgeStorage
 * @dev A contract to store and manage cross-chain transfer signatures and details.
 */
contract BridgeStorage {
    // Current epoch for each chain
    mapping(string => uint256) public chainEpoch;

    // Current epoch for each validator
    mapping(string => uint256) public validatorEpoch;

    // Mapping from staker's address to an array of their signatures.
    mapping(string => mapping(string => SignerAndSignature[]))
        public stakingSignatures;

    // Mapping of existing validators.
    mapping(string => bool) public validators;
    // stakingSignatures[][chainSymbol][SignerAndSignature]

    // Mapping of votes of new validators. validatorStatusChangeVotes[validatorAddress][status][validatorEpoch] = numberOfVotes
    mapping(string => mapping(bool => mapping(uint256 => uint256)))
        public validatorStatusChangeVotes;

    // Mapping to check if already voted on newValidator  validatorVoted[validatorAddress][senderAddress][validatorEpoch] = true/ false
    mapping(string => mapping(address => mapping(uint256 => bool)))
        public validatorVoted;

    // Mapping of votes of new chain fee. chainFeeVotes[chain][fee][chainEpoch] = votes
    mapping(string => mapping(uint256 => mapping(uint256 => uint256)))
        public chainFeeVotes;

    // Mapping of to check if already voted on a chain fee. chainFeeVotes[chain][fee][address][chainEpoch] = true/false
    mapping(string => mapping(uint256 => mapping(address => mapping(uint256 => bool))))
        public chainFeeVoted;

    // Mapping of to check if already voted on a chain fee. chainFeeVotes[chain][receiver][address][chainEpoch] = true/false
    mapping(string => mapping(string => mapping(address => mapping(uint256 => bool))))
        public chainRoyaltyVoted;

    // Mapping of votes of new chain fee. chainFeeVotes[chain][receiver][chainEpoch] = votes
    mapping(string => mapping(string => mapping(uint256 => uint256)))
        public chainRoyaltyVotes;

    // total validator count
    uint256 public validatorCount;

    // Mapping lockSignatures[txHAsh][Chain] => signatures array
    mapping(string => mapping(string => SignerAndSignature[]))
        public lockSignatures;

    // Mapping to check if a signature has already been used.
    mapping(string => bool) public usedSignatures;

    // Mapping to store fee for all chains
    mapping(string => Chain) public chainFee;

    // Mapping to store royalty receiver and percentage
    // royalties
    // mapping(string => Royalty) public royalties;

    /**
     * @dev bootstrap the bridge storage with a intial validator and intial fee for chains
     * @param _bootstrapValidator Address of the staker.
     * @param _bootstrapChainFee array of chain fee
     */
    constructor(
        string memory _bootstrapValidator,
        ChainFee[] memory _bootstrapChainFee // Royalty[] memory _royalties
    ) {
        validators[_bootstrapValidator] = true;
        validatorCount++;

        for (uint256 i = 0; i < _bootstrapChainFee.length; i++) {
            chainFee[_bootstrapChainFee[i].chain].fee = _bootstrapChainFee[i]
                .fee;

            chainFee[_bootstrapChainFee[i].chain]
                .royaltyReceiver = _bootstrapChainFee[i].royaltyReceiver;
        }

        // for (uint256 i = 0; i < _royalties.length; i++) {
        //     royalties[_bootstrapValidator].receiver = _royalties[i].receiver;
        //     royalties[_bootstrapValidator].amount = _royalties[i].amount;
        // }
    }

    /**
     * @dev modifier to check if caller is validator
     */
    modifier onlyValidator() {
        require(
            validators[addressToString(msg.sender)],
            "Only validators can call this function"
        );
        _;
    }

    /**
     * @dev change / add a chain with new fee
     * @param _chain  new / old chain.
     * @param _fee  new fee.
     */
    function changeChainFee(
        string calldata _chain,
        uint256 _fee
    ) public onlyValidator {
        require(
            chainFeeVoted[_chain][_fee][msg.sender][chainEpoch[_chain]] ==
                false,
            "Already voted"
        );
        chainFeeVoted[_chain][_fee][msg.sender][chainEpoch[_chain]] = true;

        chainFeeVotes[_chain][_fee][chainEpoch[_chain]]++;

        uint256 twoByThreeValidators = (2 * validatorCount) / 3;

        if (
            chainFeeVotes[_chain][_fee][chainEpoch[_chain]] >=
            twoByThreeValidators + 1
        ) {
            chainFee[_chain].fee = _fee;
            chainEpoch[_chain]++;
        }
    }

    /**
     * @dev change / add a chain with new fee
     * @param _chain  new / old chain.
     * @param _royaltyReceiver  new royalty receiver.
     */
    function changeChainRoyalty(
        string calldata _chain,
        string memory _royaltyReceiver
    ) public onlyValidator {
        require(
            chainRoyaltyVoted[_chain][_royaltyReceiver][msg.sender][
                chainEpoch[_chain]
            ] == false,
            "Already voted"
        );
        chainRoyaltyVoted[_chain][_royaltyReceiver][msg.sender][
            chainEpoch[_chain]
        ] = true;

        chainRoyaltyVotes[_chain][_royaltyReceiver][chainEpoch[_chain]]++;

        uint256 twoByThreeValidators = (2 * validatorCount) / 3;

        if (
            chainRoyaltyVotes[_chain][_royaltyReceiver][chainEpoch[_chain]] >=
            twoByThreeValidators + 1
        ) {
            chainFee[_chain].royaltyReceiver = _royaltyReceiver;
            chainEpoch[_chain]++;
        }
    }

    /**
     * @dev change _status of a validator with  2/3 + 1 of total validators votes are given
     * @param _validatorAddress new validator address.
     * @param _status new validator address.
     */
    function changeValidatorStatus(
        string memory _validatorAddress,
        bool _status
    ) public onlyValidator {
        uint256 _validatorEpoch = validatorEpoch[_validatorAddress];
        require(
            validatorVoted[_validatorAddress][msg.sender][_validatorEpoch] ==
                false,
            "Already voted for this validator"
        );

        validatorVoted[_validatorAddress][msg.sender][_validatorEpoch] = true;

        validatorStatusChangeVotes[_validatorAddress][_status][
            _validatorEpoch
        ]++;

        uint256 twoByThreeValidators = (2 * validatorCount) / 3;

        uint256 votes = validatorStatusChangeVotes[_validatorAddress][_status][
            _validatorEpoch
        ];

        if (votes >= twoByThreeValidators + 1) {
            // console.log("INSIDE");
            if (_status && validators[_validatorAddress] == false)
                validatorCount++;
            else if (_status == false && validators[_validatorAddress])
                validatorCount--;
            validators[_validatorAddress] = _status;
            validatorEpoch[_validatorAddress]++;
        }
    }

    /**
     * @dev Approves a stake using a signature.
     * @param _stakerAddress Address of the staker.
     * @param _signature The signature to be approved.
     */
    function approveStake(
        string calldata _stakerAddress,
        string calldata _signature,
        string calldata _chainSymbol,
        string calldata _signerAddress
    ) public onlyValidator {
        require(!usedSignatures[_signature], "Signature already used");
        usedSignatures[_signature] = true;
        SignerAndSignature memory signerAndSignature;
        signerAndSignature.publicAddress = _signerAddress;
        signerAndSignature.signature = _signature;
        stakingSignatures[_stakerAddress][_chainSymbol].push(
            signerAndSignature
        );
        changeValidatorStatus(_stakerAddress, true);
    }

    /**
     * @dev Retrieves staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Array of signatures.
     */
    function getStakingSignatures(
        string memory stakerAddress,
        string memory chainSymbol
    ) external view returns (SignerAndSignature[] memory) {
        return stakingSignatures[stakerAddress][chainSymbol];
    }

    /**
     * @dev Retrieves the count of staking signatures for a specific staker.
     * @param stakerAddress Address of the staker.
     * @return Number of signatures.
     */
    function getStakingSignaturesCount(
        string calldata stakerAddress,
        string calldata chainSymbol
    ) external view returns (uint256) {
        return stakingSignatures[stakerAddress][chainSymbol].length;
    }

    /**
     * @dev Approves the locking of an NFT using a signature.
     * @param _transactionHash tx hash of source chain
     * @param _chain source chain name eg BSC
     * @param _signature The signature to be approved.
     */
    function approveLockNft(
        string calldata _transactionHash,
        string calldata _chain,
        string calldata _signature,
        string calldata _signerAddress
    ) public onlyValidator {
        require(!usedSignatures[_signature], "Signature already used");
        usedSignatures[_signature] = true;
        SignerAndSignature memory signerAndSignatre;
        signerAndSignatre.publicAddress = _signerAddress;
        signerAndSignatre.signature = _signature;
        lockSignatures[_transactionHash][_chain].push(signerAndSignatre);
    }

    /**
     * @dev Retrieves lock signatures for a specific chain and transaction.
     * @param transactionHash tx hash of source chain
     * @param chain source chain name eg BSC
     * @return signatures array.
     */
    function getLockNftSignatures(
        string calldata transactionHash,
        string calldata chain
    ) external view returns (SignerAndSignature[] memory) {
        return lockSignatures[transactionHash][chain];
    }

    /**
     * @dev Retrieves the count of lock signatures for a specific chain and transaction.
     * @param transactionHash tx hash of source chain
     * @param chain source chain name eg BSC
     * @return Number of signatures.
     */
    function getLockNftSignaturesCount(
        string calldata transactionHash,
        string calldata chain
    ) external view returns (uint256) {
        return lockSignatures[transactionHash][chain].length;
    }

    /**
     * @dev Converts an Ethereum address to its string representation.
     * @param _address The Ethereum address to convert.
     * @return The string representation of the Ethereum address.
     */
    function addressToString(
        address _address
    ) internal pure returns (string memory) {
        bytes32 value = bytes32(uint256(uint160(_address))); // Corrected casting
        bytes memory alphabet = "0123456789abcdef";

        bytes memory str = new bytes(42);
        str[0] = 0x30; // '0'
        str[1] = 0x78; // 'x'
        for (uint256 i = 0; i < 20; i++) {
            str[2 + i * 2] = alphabet[uint8(value[i + 12] >> 4)];
            str[3 + i * 2] = alphabet[uint8(value[i + 12] & 0x0f)];
        }
        return string(str);
    }
}
