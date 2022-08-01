package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkErrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/gaia/v7/x/hal/types"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"time"
)

var _ types.MsgServer = (*msgServer)(nil)

type msgServer struct {
	Keeper
}

func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

func (k msgServer) MintHAL(goCtx context.Context, req *types.MsgMintHAL) (*types.MsgMintHALResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}
	ctx := sdk.UnwrapSDKContext(goCtx)

	accAddr, err := sdk.AccAddressFromBech32(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "address parsing: %v", err)
	}

	halCoin, colUsedCoins, err := k.ConvertCollateralsToHAL(ctx, req.CollateralAmount)
	if err != nil {
		return nil, err
	}
	if halCoin.IsZero() {
		return nil, sdkErrors.Wrapf(sdkErrors.ErrInsufficientFunds, "can not mint HAL tokens with provided collaterals")
	}

	if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, types.ActivePoolName, colUsedCoins); err != nil {
		return nil, err
	}

	if err := k.bankKeeper.MintCoins(ctx, types.ActivePoolName, sdk.NewCoins(halCoin)); err != nil {
		return nil, sdkErrors.Wrapf(types.ErrInternal, "minting HAL coin (%s): %v", halCoin, err)
	}

	if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ActivePoolName, accAddr, sdk.NewCoins(halCoin)); err != nil {
		return nil, sdkErrors.Wrapf(types.ErrInternal, "sending HAL coin (%s) from module to account: %v", halCoin, err)
	}

	ctx.EventManager().EmitEvent(
		types.NewMintEvent(accAddr, halCoin, colUsedCoins),
	)

	return &types.MsgMintHALResponse{
		MintedAmount:      halCoin,
		CollateralsAmount: colUsedCoins,
	}, nil
}

func (k msgServer) RedeemCollateral(goCtx context.Context, req *types.MsgRedeemCollateral) (*types.MsgRedeemCollateralResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}
	ctx := sdk.UnwrapSDKContext(goCtx)

	accAddr, err := sdk.AccAddressFromBech32(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "address parsing: %v", err)
	}

	halUsedCoin, colCoins, err := k.ConvertHALToCollaterals(ctx, req.HalAmount)
	if err != nil {
		return nil, err
	}
	if colCoins.IsZero() {
		return nil, sdkErrors.Wrapf(sdkErrors.ErrInsufficientFunds, "HAL amount is too small or pool funds are insufficient")
	}

	if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ActivePoolName, accAddr, colCoins); err != nil {
		return nil, sdkErrors.Wrapf(types.ErrInternal, "transferring collateral coins (%s) between pools: %v", colCoins, err)
	}


	completionTime := time.Now() 
	return &types.MsgRedeemCollateralResponse{
		BurnedAmount:   halUsedCoin,
		RedeemedAmount: colCoins,
		CompletionTime: completionTime,
	}, nil
}
