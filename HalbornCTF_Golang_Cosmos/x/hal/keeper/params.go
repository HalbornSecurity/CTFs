package keeper

import (
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/gaia/v7/x/hal/types"
)

// RedeemDur returns HAL -> collateral coins redeem timeout duration.
func (k Keeper) RedeemDur(ctx sdk.Context) (res time.Duration) {
	k.paramStore.Get(ctx, types.ParamsKeyRedeemDur, &res)
	return
}

// MaxRedeemEntries returns the max number of redeem entries per account.
func (k Keeper) MaxRedeemEntries(ctx sdk.Context) (res uint32) {
	k.paramStore.Get(ctx, types.ParamsKeyMaxRedeemEntries, &res)
	return
}

// CollateralMetas returns supported collateral token metas.
func (k Keeper) CollateralMetas(ctx sdk.Context) (res []types.TokenMeta) {
	k.paramStore.Get(ctx, types.ParamsKeyCollateralMetas, &res)
	return
}

// BaseMeta returns meta with the minimum decimals amount (to normalize coins).
func (k Keeper) BaseMeta(ctx sdk.Context) types.TokenMeta {
	halMeta := k.HALMeta(ctx)
	minMeta := halMeta

	for _, meta := range k.CollateralMetas(ctx) {
		if meta.Decimals > minMeta.Decimals {
			minMeta = meta
		}
	}

	return minMeta
}

// CollateralMetasSet returns supported collateral token metas set (key: denom).
func (k Keeper) CollateralMetasSet(ctx sdk.Context) map[string]types.TokenMeta {
	metas := k.CollateralMetas(ctx)

	set := make(map[string]types.TokenMeta, len(metas))
	for _, meta := range metas {
		set[meta.Denom] = meta
	}

	return set
}

// HALMeta returns the HAL token meta.
func (k Keeper) HALMeta(ctx sdk.Context) (res types.TokenMeta) {
	k.paramStore.Get(ctx, types.ParamsKeyHALMeta, &res)
	return
}

// GetParams returns all module parameters.
func (k Keeper) GetParams(ctx sdk.Context) types.Params {
	return types.NewParams(
		k.RedeemDur(ctx),
		k.MaxRedeemEntries(ctx),
		k.CollateralMetas(ctx),
		k.HALMeta(ctx),
	)
}

// SetParams sets all module parameters.
func (k Keeper) SetParams(ctx sdk.Context, params types.Params) {
	k.paramStore.SetParamSet(ctx, &params)
}
