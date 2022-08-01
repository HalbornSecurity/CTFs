package types

import (
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	EventTypeMint         = "hal_minted"
	EventTypeRedeemQueued = "collateral_redeem_queued"
	EventTypeRedeemDone   = "collateral_redeem_done"

	AttributeKeyMintedAmount   = "minted_amount"
	AttributeKeyRedeemedAmount = "redeemed_amount"
	AttributeKeyUsedAmount     = "used_amount"
	AttributeKeyCompletionTime = "completion_time"
)

// NewMintEvent creates a new HAL mint event.
func NewMintEvent(accAddr sdk.AccAddress, mintedHALCoin sdk.Coin, usedCollateralCoins sdk.Coins) sdk.Event {
	return sdk.NewEvent(
		EventTypeMint,
		sdk.NewAttribute(sdk.AttributeKeySender, accAddr.String()),
		sdk.NewAttribute(AttributeKeyMintedAmount, mintedHALCoin.String()),
		sdk.NewAttribute(AttributeKeyUsedAmount, usedCollateralCoins.String()),
	)
}

// NewRedeemQueuedEvent creates a new redeem enqueue event.
func NewRedeemQueuedEvent(accAddr sdk.AccAddress, usedHALCoin sdk.Coin, redeemedCollateralCoins sdk.Coins, completionTime time.Time) sdk.Event {
	return sdk.NewEvent(
		EventTypeRedeemQueued,
		sdk.NewAttribute(sdk.AttributeKeySender, accAddr.String()),
		sdk.NewAttribute(AttributeKeyUsedAmount, usedHALCoin.String()),
		sdk.NewAttribute(AttributeKeyRedeemedAmount, redeemedCollateralCoins.String()),
		sdk.NewAttribute(AttributeKeyCompletionTime, completionTime.String()),
	)
}

// NewRedeemDoneEvent creates a new redeem dequeue event.
func NewRedeemDoneEvent(accAddr sdk.AccAddress, amount sdk.Coins, completionTime time.Time) sdk.Event {
	return sdk.NewEvent(
		EventTypeRedeemDone,
		sdk.NewAttribute(sdk.AttributeKeySender, accAddr.String()),
		sdk.NewAttribute(sdk.AttributeKeyAmount, amount.String()),
		sdk.NewAttribute(AttributeKeyCompletionTime, completionTime.String()),
	)
}
