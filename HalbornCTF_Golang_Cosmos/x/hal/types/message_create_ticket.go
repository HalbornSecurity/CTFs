package types

import (
	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ sdk.Msg = &MsgCreateTicket{}

func NewMsgCreateTicket(creator string, author string, issue string) *MsgCreateTicket {
	return &MsgCreateTicket{
		Creator: creator,
		Author:  author,
		Issue:   issue,
	}
}

func (msg *MsgCreateTicket) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Creator)
	if err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid creator address (%s)", err)
	}
	return nil
}
