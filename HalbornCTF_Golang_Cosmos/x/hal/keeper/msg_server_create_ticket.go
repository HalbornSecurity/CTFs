package keeper

import (
	"context"

	"HalbornCTF/x/hal/types"

	errorsmod "cosmossdk.io/errors"
	math "cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Create a ticket paying 1 Hal token per sigle letter
func (k msgServer) CreateTicket(goCtx context.Context, msg *types.MsgCreateTicket) (*types.MsgCreateTicketResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	accAddr, err := sdk.AccAddressFromBech32(msg.Creator)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid creator address (%s)", err)
	}

	// Calculate the cost based on the issue length (1 HAL per letter)
	issueLength := len(msg.Issue)
	halCost := math.NewUint(uint64(issueLength))

	params := k.GetParams(ctx)
	halDenom := params.HalMeta.Denom
	halCoin := sdk.NewCoin(halDenom, math.Int(halCost))

	if !k.bankKeeper.HasBalance(ctx, accAddr, halCoin) {
		return nil, errorsmod.Wrap(sdkerrors.ErrInsufficientFunds, "insufficient HAL tokens")
	}

	if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, types.TreasuryKey, sdk.NewCoins(halCoin)); err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInsufficientFunds, "failed to deduct HAL tokens")
	}

	// Create and store the ticket
	var ticket = types.Ticket{
		Author: msg.Author,
		Issue:  msg.Issue,
	}

	id := k.AppendTicket(
		ctx,
		ticket,
	)

	return &types.MsgCreateTicketResponse{
		Id: id,
	}, nil
}
