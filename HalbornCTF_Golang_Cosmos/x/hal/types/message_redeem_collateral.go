package types

import (
	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ sdk.Msg = &MsgRedeemCollateral{}

func NewMsgRedeemCollateral(creator string, halAmount sdk.Coin) *MsgRedeemCollateral {
	return &MsgRedeemCollateral{
		Creator:   creator,
		HalAmount: halAmount,
	}
}

func (msg *MsgRedeemCollateral) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Creator)
	if err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid creator address (%s)", err)
	}
	return nil
}
