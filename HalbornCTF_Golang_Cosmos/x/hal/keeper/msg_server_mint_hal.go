package keeper

import (
	"context"
	"fmt"

	"HalbornCTF/x/hal/types"

	errorsmod "cosmossdk.io/errors"
	math "cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func (k msgServer) MintHal(goCtx context.Context, msg *types.MsgMintHal) (*types.MsgMintHalResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	accAddr, err := sdk.AccAddressFromBech32(msg.Creator)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid creator address (%s)", err)
	}

	if !msg.CollateralAmount.IsValid() && msg.CollateralAmount.IsZero() {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "collateral amount is invalid or zero")
	}

	params := k.GetParams(ctx)
	collateralDenom := params.CollateralMetas.Denom

	if msg.CollateralAmount.Denom != collateralDenom {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, fmt.Sprintf("collateral denom mismatch: expected %s, got %s", collateralDenom, msg.CollateralAmount.Denom))
	}

	//Define the exchange rate
	exchangeRate := math.LegacyNewDec(2) // 1 COLL -> 2 HAL

	//Calculate the amount of HAL to mint
	collateralAmount := math.LegacyNewDecFromInt(msg.CollateralAmount.Amount) // Convert collateral to decimal
	mintedHalAmount := collateralAmount.Mul(exchangeRate).TruncateInt()

	// Retrieve token denom
	halDenom := params.HalMeta.Denom

	mintedHalCoin := sdk.NewCoin(halDenom, mintedHalAmount)

	//Mint the HAL tokens to the module account
	if err := k.bankKeeper.MintCoins(ctx, types.CollateralPoolName, sdk.NewCoins(mintedHalCoin)); err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInsufficientFunds, "failed to mint HAL coins")
	}

	if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.CollateralPoolName, accAddr, sdk.NewCoins(mintedHalCoin)); err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInsufficientFunds, "failed to send minted HAL coins to creator")
	}

	k.bankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, types.CollateralPoolName, sdk.NewCoins(msg.CollateralAmount))

	return &types.MsgMintHalResponse{
		MintedAmount: mintedHalCoin,
	}, nil
}
