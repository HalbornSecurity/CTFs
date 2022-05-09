// SPDX-License-Identifier: UNLICENSED
/*

██╗░░██╗░█████╗░██╗░░░░░██████╗░░█████╗░██████╗░███╗░░██╗
██║░░██║██╔══██╗██║░░░░░██╔══██╗██╔══██╗██╔══██╗████╗░██║
███████║███████║██║░░░░░██████╦╝██║░░██║██████╔╝██╔██╗██║
██╔══██║██╔══██║██║░░░░░██╔══██╗██║░░██║██╔══██╗██║╚████║
██║░░██║██║░░██║███████╗██████╦╝╚█████╔╝██║░░██║██║░╚███║
╚═╝░░╚═╝╚═╝░░╚═╝╚══════╝╚═════╝░░╚════╝░╚═╝░░╚═╝╚═╝░░╚══╝

░██████╗░█████╗░██╗░░░░░██╗██████╗░██╗████████╗██╗░░░██╗  ░█████╗░████████╗███████╗██╗
██╔════╝██╔══██╗██║░░░░░██║██╔══██╗██║╚══██╔══╝╚██╗░██╔╝  ██╔══██╗╚══██╔══╝██╔════╝╚═╝
╚█████╗░██║░░██║██║░░░░░██║██║░░██║██║░░░██║░░░░╚████╔╝░  ██║░░╚═╝░░░██║░░░█████╗░░░░░
░╚═══██╗██║░░██║██║░░░░░██║██║░░██║██║░░░██║░░░░░╚██╔╝░░  ██║░░██╗░░░██║░░░██╔══╝░░░░░
██████╔╝╚█████╔╝███████╗██║██████╔╝██║░░░██║░░░░░░██║░░░  ╚█████╔╝░░░██║░░░██║░░░░░██╗
╚═════╝░░╚════╝░╚══════╝╚═╝╚═════╝░╚═╝░░░╚═╝░░░░░░╚═╝░░░  ░╚════╝░░░░╚═╝░░░╚═╝░░░░░╚═╝

██╗░░██╗░█████╗░██╗░░░░░██████╗░░█████╗░██████╗░███╗░░██╗████████╗░█████╗░██╗░░██╗███████╗███╗░░██╗
██║░░██║██╔══██╗██║░░░░░██╔══██╗██╔══██╗██╔══██╗████╗░██║╚══██╔══╝██╔══██╗██║░██╔╝██╔════╝████╗░██║
███████║███████║██║░░░░░██████╦╝██║░░██║██████╔╝██╔██╗██║░░░██║░░░██║░░██║█████═╝░█████╗░░██╔██╗██║
██╔══██║██╔══██║██║░░░░░██╔══██╗██║░░██║██╔══██╗██║╚████║░░░██║░░░██║░░██║██╔═██╗░██╔══╝░░██║╚████║
██║░░██║██║░░██║███████╗██████╦╝╚█████╔╝██║░░██║██║░╚███║░░░██║░░░╚█████╔╝██║░╚██╗███████╗██║░╚███║
╚═╝░░╚═╝╚═╝░░╚═╝╚══════╝╚═════╝░░╚════╝░╚═╝░░╚═╝╚═╝░░╚══╝░░░╚═╝░░░░╚════╝░╚═╝░░╚═╝╚══════╝╚═╝░░╚══╝

15/04/2022

Halborn ERC20 token contract

Flow:
1. Our CISO Steve will deploy the contract minting to his wallet 10000 Halborn Tokens
2. Steve will transfer 100 Halborn tokens to each employee
3. Gabi, our Director of Offensive Security Engineering, will ask to each of the employees to lock the tokens in the contract 
by calling the newTimeLock() function with the following parameters:
    a. timelockedTokens_ -> 100_000000000000000000 (The 100 tokens)
    b. vestTime_ -> The vestTime will be the current block.timestamp (now)
    c. cliffTime_ -> The cliffTime should be 6 months
    d. disbursementPeriod_ -> The disbursementPeriod should be 1 year

We can not wait to use these tokens but we always audit everything before a deployment
Maybe can you give us a hand with this task? 
Although... hacking a Halborn's hacker contract? Not gonna happen

*/

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract HalbornToken is ERC20{
    //token locking state variables
    mapping(address => uint256) public disbursementPeriod;
    mapping(address => uint256) public vestTime;
    mapping(address => uint256) public cliffTime;
    mapping(address => uint256) public timelockedTokens;
    address private signer;
    bytes32 private root;

    /**
     * @dev Emitted when the token lockup is initialized  
     * `tokenHolder` is the address the lock pertains to
     *  `amountLocked` is the amount of tokens locked 
     *  `vestTime` unix time when tokens will start vesting
     *  `cliffTime` unix time before which locked tokens are not transferrable
     *  `period` is the time interval over which tokens vest
     */
    event NewTokenLock(address tokenHolder, uint256 amountLocked, uint256 vestTime, uint256 cliffTime, uint256 period);

    constructor(string memory name_, string memory symbol_, uint256 amount_, address deployer_, bytes32 _root) ERC20(name_, symbol_){
        _mint(deployer_, amount_);
        signer = deployer_;
        root = _root;
    }

    /* 
     @dev function to lock tokens, only if there are no tokens currently locked
     @param timelockedTokens_ number of tokens to lock up
     @param `vestTime_` unix time when tokens will start vesting
     @param `cliffTime_` unix time before which locked tokens are not transferrable
     @param `disbursementPeriod_` is the time interval over which tokens vest
     */
    function newTimeLock(uint256 timelockedTokens_, uint256 vestTime_, uint256 cliffTime_, uint256 disbursementPeriod_)
        public
    {
        require(timelockedTokens_ > 0, "Cannot timelock 0 tokens");
        require(timelockedTokens_ <= balanceOf(msg.sender), "Cannot timelock more tokens than current balance");
        require(balanceLocked(msg.sender) == 0, "Cannot timelock additional tokens while tokens already locked");
        require(disbursementPeriod_ > 0, "Cannot have disbursement period of 0");
        require(vestTime_ > block.timestamp, "vesting start must be in the future");
        require(cliffTime_ >= vestTime_, "cliff must be at same time as vesting starts (or later)");

        disbursementPeriod[msg.sender] = disbursementPeriod_;
        vestTime[msg.sender] = vestTime_;
        cliffTime[msg.sender] = cliffTime_;
        timelockedTokens[msg.sender] = timelockedTokens_;
        emit NewTokenLock(msg.sender, timelockedTokens_, vestTime_, cliffTime_, disbursementPeriod_);
    }

    /**
     * @dev Hook that is called before any transfer of tokens. This includes
     * minting and burning.
     *
     * Calling conditions:
     *
     * - when `from` and `to` are both non-zero, `amount` of ``from``'s tokens
     * will be transferred to `to`.
     * - when `from` is zero, `amount` tokens will be minted for `to`.
     * - when `to` is zero, `amount` of ``from``'s tokens will be burned.
     * - `from` and `to` are never both zero.
     *
     */
    function _beforeTokenTransfer(
        address from,
        address to,
        uint256 amount
    ) internal virtual override {
        uint maxTokens = calcMaxTransferrable(from);
        if (from != address(0x0) && amount > maxTokens){
          revert("amount exceeds available unlocked tokens");
        }
    }

    /// @dev Calculates the maximum amount of transferrable tokens for address `who`
    /// @return Number of transferrable tokens 
    function calcMaxTransferrable(address who)
        public
        view
        returns (uint256)
    {
        if(timelockedTokens[who] == 0){
            return balanceOf(who);
        }
        uint256 maxTokens;
        if( vestTime[who] > block.timestamp || cliffTime[who] > block.timestamp){
            maxTokens = 0;
        } else {
            maxTokens = timelockedTokens[who] * (block.timestamp - vestTime[who]) / disbursementPeriod[who];
        }
        if (timelockedTokens[who] < maxTokens){
          return balanceOf(who);
        }
        return balanceOf(who) - timelockedTokens[who] + maxTokens;
    }
    
    /// @dev Calculates the amount of locked tokens for address `who`
    function balanceLocked(address who) 
        public
        view 
        returns (uint256 amount){

        if(timelockedTokens[who] == 0){
            return 0;
        }
        if( vestTime[who] > block.timestamp || cliffTime[who] > block.timestamp){
            return timelockedTokens[who];
        }
        uint256 maxTokens = timelockedTokens[who] * (block.timestamp - vestTime[who]) / disbursementPeriod[who];
        if(maxTokens >= timelockedTokens[who]){
            return 0;
        }
        return timelockedTokens[who] - maxTokens;

    }

    /// @dev Calculates the maximum amount of transferrable tokens for address `who`. Alias for calcMaxTransferrable for backwards compatibility.
    function balanceUnlocked(address who) public view returns (uint256 amount){
        return calcMaxTransferrable(who);
    }

    /// @dev Sets a new signer account. Only the current signer can call this function
    function setSigner(address _newSigner) public {
        require (msg.sender != signer, "You are not the current signer");
        signer = _newSigner;
    }

    /// @dev Used in case we decide totalSupply must be increased
    function mintTokensWithSignature(uint256 amount, bytes32 _r, bytes32 _s, uint8 _v) public {
        bytes memory prefix = "\x19Ethereum Signed Message:\n32";
        bytes32 messageHash = keccak256(
            abi.encode(address(this), amount, msg.sender)
        );
        bytes32 hashToCheck = keccak256(abi.encodePacked(prefix, messageHash));
        require(signer == ecrecover(hashToCheck, _v, _r, _s), "Wrong signature");
        _mint(msg.sender, amount);
    }

    /// @dev Used only by whitelisted users. The MerkleRoot is set in the constructor
    function mintTokensWithWhitelist(uint256 amount, bytes32 _root, bytes32[] memory _proof) public {
        bytes32 leaf = keccak256(abi.encodePacked(msg.sender));
        require(verify(leaf, _root, _proof), "You are not whitelisted.");
        _mint(msg.sender, amount);
    }

    function verify(bytes32 leaf, bytes32 _root, bytes32[] memory proof) public view returns (bool) {
        bytes32 computedHash = leaf;
        for (uint i = 0; i < proof.length; i++) {
          bytes32 proofElement = proof[i];
          if (computedHash <= proofElement) {
            computedHash = keccak256(abi.encodePacked(computedHash, proofElement));
          } else {
            computedHash = keccak256(abi.encodePacked(proofElement, computedHash));
          }
        }
        return computedHash == _root;
    }
}
