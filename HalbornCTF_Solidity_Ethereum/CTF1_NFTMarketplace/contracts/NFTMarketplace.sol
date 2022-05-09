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

███╗░░██╗███████╗████████╗███╗░░░███╗░█████╗░██████╗░██╗░░██╗███████╗████████╗██████╗░██╗░░░░░░█████╗░░█████╗░███████╗
████╗░██║██╔════╝╚══██╔══╝████╗░████║██╔══██╗██╔══██╗██║░██╔╝██╔════╝╚══██╔══╝██╔══██╗██║░░░░░██╔══██╗██╔══██╗██╔════╝
██╔██╗██║█████╗░░░░░██║░░░██╔████╔██║███████║██████╔╝█████═╝░█████╗░░░░░██║░░░██████╔╝██║░░░░░███████║██║░░╚═╝█████╗░░
██║╚████║██╔══╝░░░░░██║░░░██║╚██╔╝██║██╔══██║██╔══██╗██╔═██╗░██╔══╝░░░░░██║░░░██╔═══╝░██║░░░░░██╔══██║██║░░██╗██╔══╝░░
██║░╚███║██║░░░░░░░░██║░░░██║░╚═╝░██║██║░░██║██║░░██║██║░╚██╗███████╗░░░██║░░░██║░░░░░███████╗██║░░██║╚█████╔╝███████╗
╚═╝░░╚══╝╚═╝░░░░░░░░╚═╝░░░╚═╝░░░░░╚═╝╚═╝░░╚═╝╚═╝░░╚═╝╚═╝░░╚═╝╚══════╝░░░╚═╝░░░╚═╝░░░░░╚══════╝╚═╝░░╚═╝░╚════╝░╚══════╝

13/04/2022

This contract is used by Halborn employees to sell their awesome NFTs (https://opensea.io/collection/halbornteam). 
Nah, not really, these NFTs are too cool to be sold and apart from that we have been told that this smart contract
is full of bugs.

Can you help us to fix it?

*/

import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import "@openzeppelin/contracts/utils/Context.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

pragma solidity ^0.8.0;

contract NFTMarketplace is AccessControlEnumerable, ReentrancyGuard, IERC721Receiver
{
    using Counters for Counters.Counter;

    enum OrderStatus {
        Listed,
        Fulfilled,
        Cancelled
    }

    struct Order {
        address owner;
        uint256 amount;
        uint256 nftId;
        OrderStatus status;
    }

    struct Bid {
        address owner;
        uint256 amount;
    }

    // Roles
    bytes32 public immutable ADMIN_ROLE;
    bytes32 public immutable ADMIN_ROLE_ADMIN;

    /** Contract
        1. ApeCoin: ERC20 contract used as the payment currency in the NFTmarketplace
        2. HalbornNFTcollection: ERC721 contract whose NFTs will be traded in this NFTmarketplace.
    */
    IERC20 public immutable ApeCoin;
    IERC721 public immutable HalbornNFTcollection;

    // NftId => order Ids; dapp usage mainly
    mapping(uint256 => uint256[]) public buyOrderIds;

    // Order Id => Order
    mapping(uint256 => Order) public buyOrders;

    // Id tracker
    Counters.Counter private buyOrderCounter;

    // Sell order: HalbornNFTcollectionID -> Order
    mapping(uint256 => Order) public sellOrder;

    // nftId => Bid
    mapping(uint256 => Bid) public bidOrders;

    event BuyOrderListed(
        address indexed owner,
        uint256 indexed orderId,
        uint256 indexed nftId,
        uint256 erc20Amount
    );

    event BuyOrderFulfilled(uint256 indexed orderId);

    event BuyOrderCancelled(uint256 indexed orderId);

    event BuyOrderIncreased(
        uint256 indexed orderId,
        uint256 indexed increaseAmount
    );

    event BuyOrderDecreased(
        uint256 indexed orderId,
        uint256 indexed decreaseAmount
    );

    event SellOrderListed(
        address indexed owner,
        uint256 indexed nftId,
        uint256 indexed erc20Amount
    );

    event SellOrderFulfilled(
        uint256 indexed nftId,
        uint256 indexed erc20Amount
    );

    event SellOrderCancelled(
        uint256 indexed nftId,
        uint256 indexed erc20Amount
    );

    event ERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes data,
        uint256 gas
    );

    constructor(
        address governance,
        address erc20token,
        address nftToken
    ) {
        require((governance != address(0)) && (erc20token != address(0)) && (nftToken != address(0)), "NFTMarketplace: address(0)");
        // Contracts
        ApeCoin = IERC20(erc20token);
        HalbornNFTcollection = IERC721(nftToken);
        // Roles
        ADMIN_ROLE = keccak256("ADMIN_ROLE");
        ADMIN_ROLE_ADMIN = keccak256("ADMIN_ROLE_ADMIN");
        // setup role admins
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE_ADMIN);
        // grant admins
        _setupRole(ADMIN_ROLE_ADMIN, governance);
        // grant roles
        _setupRole(ADMIN_ROLE, governance);
    }

    /**
     *
     * @dev Lists a buy order for a given user by taking the ApeCoins as collateral to fulfill the order
     *
    */
    function postBuyOrder(uint256 nftId, uint256 erc20AmountOffer)
        external
        nonReentrant
    {
        // require existence of the nftId
        require(
            HalbornNFTcollection.ownerOf(nftId) != address(0),
            "nftID does not exists"
        );
        // require that the erc20AmountOffer is greater than 0
        require(erc20AmountOffer > 0, "Offer should be > 0");
        // transfer ApeCoins to the contract
        require(
            ApeCoin.transferFrom(_msgSender(), address(this), erc20AmountOffer),
            "ApeCoin transfer failed"
        );
        // order creation
        Order storage order = buyOrders[buyOrderCounter.current()];
        order.owner = _msgSender();
        order.amount = erc20AmountOffer;
        order.nftId = nftId;
        order.status = OrderStatus.Listed;
        // add order to global tracker
        buyOrders[buyOrderCounter.current()] = order;
        // add to the list
        buyOrderIds[nftId].push(buyOrderCounter.current());
        // emit
        emit BuyOrderListed(
            _msgSender(),
            buyOrderCounter.current(),
            nftId,
            erc20AmountOffer
        );
        // increment unique id counter
        buyOrderCounter.increment();
    }

    /**
     *
     * @dev cancels a buy order by changing its status and returning funds
     *
    */
    function cancelBuyOrder(uint256 orderId) external nonReentrant {
        Order storage order = buyOrders[orderId];
        // cannot be a cancelled or fulfilled order
        require(
            order.status != OrderStatus.Cancelled ||
                order.status != OrderStatus.Fulfilled,
            "Order should be listed"
        );
        // require the caller to be the owner of this orderId
        require(
            order.owner == _msgSender(),
            "Caller must own the buy order"
        );
        //transfer back the ApeCoin initially put as collateral
        require(
            ApeCoin.transfer(_msgSender(), order.amount),
            "ApeCoin transfer failed"
        );
        order.status = OrderStatus.Cancelled;
        emit BuyOrderCancelled(orderId);
    }

    /**
     *
     * @dev increase a buy order amount
     *
    */
    function increaseBuyOrder(uint256 orderId, uint256 increaseAmount)
        external
        nonReentrant
    {
        require(increaseAmount > 0, "increaseAmount > 0");
        Order storage order = buyOrders[orderId];
        // cannot be a cancelled or fulfilled order
        require(
            order.status != OrderStatus.Cancelled ||
                order.status != OrderStatus.Fulfilled,
            "Order should be listed"
        );
        require(
            order.owner == _msgSender(),
            "Caller must own the buy order"
        );
        require(
            ApeCoin.transferFrom(_msgSender(), address(this), increaseAmount),
            "ApeCoin transfer failed"
        );
        // increase the order
        order.amount += increaseAmount;
        emit BuyOrderIncreased(orderId, increaseAmount);
    }

    /**
     *
     * @dev decrease an existing order
     *
     */
    function decreaseBuyOrder(uint256 orderId, uint256 decreaseAmount)
        external
        nonReentrant
    {
        require(decreaseAmount > 0, "decreaseAmount > 0");
        Order storage order = buyOrders[orderId];
        require(
            order.amount > decreaseAmount,
            "order.amount > decreaseAmount"
        );
        // Can not be a cancelled or fulfilled order
        require(
            order.status != OrderStatus.Cancelled ||
                order.status != OrderStatus.Fulfilled,
            "Order should be listed"
        );
        require(
            order.owner == _msgSender(),
            "Caller must own the buy order"
        );
        require(
            ApeCoin.transfer(_msgSender(), decreaseAmount),
            "ApeCoin transfer failed"
        );
        order.amount -= decreaseAmount;
        emit BuyOrderDecreased(orderId, decreaseAmount);
    }

    /**
     *
     * @dev Creates a sell order for the HalbornNFT
     *
     * @param nftId NFT Id to sell
     * @param amount amount of ApeCoins to list for
     *
     * @notice editing this order can simply be viewed as creating a
     * whole new order since there is no reference Id
     *
    */
    function postSellOrder(uint256 nftId, uint256 amount)
        external
        nonReentrant
    {
        require(amount > 0, "amount > 0");
        // require existence of the nftId
        require(
            HalbornNFTcollection.ownerOf(nftId) != address(0),
            "nftID does not exists"
        );
        // overrides the current sellOrder
        Order storage order = sellOrder[nftId];
        order.owner = _msgSender();
        order.status = OrderStatus.Listed;
        order.amount = amount;
        order.nftId = nftId;
        // take the 721 as collateral
        HalbornNFTcollection.safeTransferFrom(
            HalbornNFTcollection.ownerOf(nftId),
            address(this),
            nftId,
            bytes("COLLATERAL")
        );
        // require balance to be 1 for the contract
        require(
            HalbornNFTcollection.ownerOf(nftId) == address(this),
            "HalbornNFTcollection: ownership"
        );
        emit SellOrderListed(_msgSender(), nftId, amount);
    }

    /**
     *
     * @dev Cancels a sell order
     *
     */
    function cancelSellOrder(uint256 nftId) external nonReentrant {
        Order storage order = sellOrder[nftId];
        // cannot be a cancelled or fulfilled order
        require(
            order.status != OrderStatus.Cancelled ||
                order.status != OrderStatus.Fulfilled,
            "Order should be listed"
        );
        // simply change status of order to cancelled
        require(
            _msgSender() == order.owner,
            "Order ownership"
        );
        // return the ERC721 NFT to the owner
        HalbornNFTcollection.safeTransferFrom(
            address(this),
            _msgSender(),
            nftId,
            bytes("RETURNING COLLATERAL")
        );
        // require ownership change
        require(
            HalbornNFTcollection.ownerOf(nftId) == _msgSender(),
            "HalbornNFTcollection: ownership 2"
        );
        order.status = OrderStatus.Cancelled;
        emit SellOrderCancelled(nftId, order.amount);
    }

    function _resetSellOrder(uint256 nftId) internal {
        Order storage order = sellOrder[nftId];
        delete order.owner;
        delete order.amount;
        delete order.nftId;
        delete order.status;
    }

    /**
     *
     * @dev purchases a HalbornNFT for the listed sell order price
     *
     * @param nftId the HalbornNFT Id to purchase
     *
     * @notice requires a valid HalbornNFT with an active sell order;
     * instantly transfers assets between buyer and seller
     *
     */
    function buySellOrder(uint256 nftId) external nonReentrant {
        Order storage order = sellOrder[nftId];
        // cannot buy your own sell order
        require(
            HalbornNFTcollection.ownerOf(nftId) != _msgSender(),
            "You can not buy your own sell order"
        );
        // transfer ApeCoins to order owner
        require(
            ApeCoin.transferFrom(_msgSender(), order.owner, order.amount),
            "ApeCoin transfer failed"
        );
        // transfer the HalbornNFT to the buyer
        HalbornNFTcollection.safeTransferFrom(
            address(this),
            _msgSender(),
            nftId,
            bytes("PURCHASE HALBORNNFT")
        );
        // require new ownership
        require(
            HalbornNFTcollection.ownerOf(nftId) == _msgSender(),
            "HalbornNFTcollection: ownership"
        );
        order.status = OrderStatus.Fulfilled;
        _resetSellOrder(nftId);
        emit SellOrderFulfilled(nftId, order.amount);
    }

    /**
     *
     * @dev used to 'sell' to a listed buy order by an owner of a HalbornNFT
     *
     * @param orderId the buy order Id to fulfill
     *
     * @notice must be a valid orderId
     *
     */
    function sellToOrderId(uint256 orderId) external nonReentrant {
        Order storage order = buyOrders[orderId];
        require(
            order.owner != _msgSender(),
            "You can not sell to yourself"
        );
        require(
            order.status != OrderStatus.Cancelled ||
                order.status != OrderStatus.Fulfilled,
            "Order should be listed"
        );
        //require the caller to own the HalbornNFT for this orderId
        require(
            HalbornNFTcollection.ownerOf(order.nftId) == _msgSender(),
            "HalbornNFTcollection: ownership"
        );
        // transfer to seller
        require(
            ApeCoin.transfer(_msgSender(), order.amount),
            "ApeCoin transfer failed"
        );
        //transfer HalbornNFT
        HalbornNFTcollection.safeTransferFrom(
            _msgSender(),
            order.owner,
            order.nftId,
            bytes("SELL ORDER")
        );
        //require new ownership
        require(
            HalbornNFTcollection.ownerOf(order.nftId) == order.owner,
            "HalbornNFTcollection: ownership"
        );
        order.status = OrderStatus.Fulfilled;
        _resetSellOrder(order.nftId);
        emit BuyOrderFulfilled(orderId);
    }

    /**
     *
     * @dev Used to 'bid' for a HalbornNFT with Ether
     *
     * @param nftId nftId of the HalbornNFTcollection
     *
     */
    function bid(uint256 nftId) external payable nonReentrant {
        require(msg.value > 0, "msg.value should be > 0");
        // require the caller to not own the nftId
        require(
            HalbornNFTcollection.ownerOf(nftId) != _msgSender(),
            "HalbornNFTcollection: ownership"
        );
        Bid storage bid = bidOrders[nftId];
        // Give back the Ether to the previous bidder
        if(bid.owner != address(0)){
            require(bid.amount < msg.value, "Your bid is not enough");
            address previousBidder = bid.owner;
            uint256 prevAmount = bid.amount;
            (bool success, ) = previousBidder.call{value: prevAmount}("");
            require(success, "Ether return for the previous bidder failed");
        }
        bid.owner = _msgSender();
        bid.amount = msg.value;
    }

    /**
     *
     * @dev Used to 'accept a Ether bid' for a HalbornNFT
     *
     * @param nftId nftId of the HalbornNFTcollection
     *
     */
    function acceptBid(uint256 nftId) external nonReentrant {
        require(
            HalbornNFTcollection.ownerOf(nftId) == _msgSender(),
            "HalbornNFTcollection: ownership"
        );
        Bid storage bid = bidOrders[nftId];
        require(bid.owner != address(0), "There is no bid for this nftId");
        //transfer HalbornNFT
        HalbornNFTcollection.safeTransferFrom(
            _msgSender(),
            bid.owner,
            nftId,
            bytes("ACCEPT BID ORDER")
        );
        (bool success, ) = _msgSender().call{value: bid.amount}("");
        require(success, "Ether payment failed");
        delete bidOrders[nftId];
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(AccessControlEnumerable)
        returns (bool)
    {
        return
            interfaceId == type(IERC721Receiver).interfaceId ||
            super.supportsInterface(interfaceId);
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes memory data
    ) public override returns (bytes4) {
        emit ERC721Received(operator, from, tokenId, data, gasleft());
        return IERC721Receiver.onERC721Received.selector;
    }

    /**
     *
     * @dev returns the enum status for a given orderId
     *
     * @param orderId the buy order to query
     *
     * @return
     *
     * Listed  - 0
     * Fulfilled  - 1
     * Canceled - 2
     *
     */
    function getOrderStatus(uint256 orderId) public view returns (OrderStatus) {
        return buyOrders[orderId].status;
    }

    /**
     *
     * @dev returns a list of orders user, amount
     *
     * @notice an rpc gas limit on eth_call will limit the return size of these arrays,
     * an estimation will cap orders at 100,000 per nftId
     *
     */
    function viewBuyOrders(uint256 nftId)
        external
        view
        returns (address[] memory, uint256[] memory)
    {
        //determine length
        uint256 length = buyOrderIds[nftId].length;
        // create fixed length arrays to return
        address[] memory addresses = new address[](length);
        uint256[] memory amounts = new uint256[](length);
        // fill the fixed length array
        for (uint256 i; i < buyOrderIds[nftId].length; ++i) {
            addresses[i] = buyOrders[buyOrderIds[nftId][i]].owner;
            amounts[i] = buyOrders[buyOrderIds[nftId][i]].amount;
        }
        // return fixed length arrays
        return (addresses, amounts);
    }

    /**
     *
     * @dev returns the most recent updated sell order
     *
     */
    function viewCurrentSellOrder(uint256 nftId)
        external
        view
        returns (address owner, uint256 amount)
    {
        return (sellOrder[nftId].owner, sellOrder[nftId].amount);
    }
}