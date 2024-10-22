package keeper

import (
	"context"
	"fmt"
	"time"

	"HalbornCTF/x/hal/types"

	errorsmod "cosmossdk.io/errors"
	math "cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func (k msgServer) RedeemCollateral(goCtx context.Context, msg *types.MsgRedeemCollateral) (*types.MsgRedeemCollateralResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	accAddr, err := sdk.AccAddressFromBech32(msg.Creator)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid creator address (%s)", err)
	}

	if !msg.HalAmount.IsValid() {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "HAL amount is invalid")
	}

	params := k.GetParams(ctx)
	halDenom := params.HalMeta.Denom

	halCoin := sdk.NewCoin(halDenom, msg.HalAmount.Amount)

	// Burn the HAL tokens from the user's account
	k.bankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, types.CollateralPoolName, sdk.NewCoins(halCoin))

	if err := k.bankKeeper.BurnCoins(ctx, types.CollateralPoolName, sdk.NewCoins(halCoin)); err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "failed to burn HAL coins")
	}

	// // Define the exchange rate (HAL to Collateral conversion)
	exchangeRate := math.LegacyNewDec(2) // 2 HAL -> 1 COLL

	// Calculate the amount of collateral to redeem
	halAmount := math.LegacyNewDecFromInt(msg.HalAmount.Amount)
	redeemedCollateralAmount := halAmount.Quo(exchangeRate).TruncateInt() // Divide by exchange rate and truncate to integer
	fmt.Println(redeemedCollateralAmount)

	collateralDenom := params.CollateralMetas.Denom
	// // Transfer the collateral tokens from CollateralPoolName to RedeemPoolName

	redeemedCollateralCoin := sdk.NewCoin(collateralDenom, redeemedCollateralAmount)
	if err := k.bankKeeper.SendCoinsFromModuleToModule(ctx, types.CollateralPoolName, types.RedeemPoolName, sdk.NewCoins(redeemedCollateralCoin)); err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInsufficientFunds, "failed to transfer collateral tokens to redeem pool")
	}

	// Set the completion time for the redeem process from params
	redeemDur := k.GetParams(ctx).RedeemDur
	completionTime := time.Now().Add(redeemDur)

	// Store the redeem request details to process at EndBlock
	redeemRequest := types.RedeemRequest{
		Account:        accAddr.String(),
		Collateral:     redeemedCollateralCoin,
		Completiontime: completionTime,
	}

	k.SetRedeemRequest(ctx, redeemRequest)

	return &types.MsgRedeemCollateralResponse{
		BurnedAmount:   halCoin,
		CompletionTime: completionTime,
	}, nil
}
