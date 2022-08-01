package hal

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/gaia/v7/x/hal/keeper"
	abci "github.com/tendermint/tendermint/abci/types"
)

// EndBlocker manages redeeming queue.
func EndBlocker(ctx sdk.Context, k keeper.Keeper) []abci.ValidatorUpdate {
	k.EndRedeeming(ctx)

	return []abci.ValidatorUpdate{}
}
