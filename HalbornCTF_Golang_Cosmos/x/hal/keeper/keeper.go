package keeper

import (
	"sort"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkErrors "github.com/cosmos/cosmos-sdk/types/errors"
	paramsTypes "github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/gaia/v7/x/hal/types"
	"github.com/tendermint/tendermint/libs/log"
)

// Keeper of the hal store.
type Keeper struct {
	storeKey   sdk.StoreKey
	cdc        codec.BinaryCodec
	authKeeper types.AccountKeeper
	bankKeeper types.BankKeeper
	paramStore paramsTypes.Subspace
}

// NewKeeper creates a new hal Keeper instance.
func NewKeeper(cdc codec.BinaryCodec, key sdk.StoreKey, ak types.AccountKeeper, bk types.BankKeeper, ps paramsTypes.Subspace) Keeper {
	// Set KeyTable if it has not already been set
	if !ps.HasKeyTable() {
		ps = ps.WithKeyTable(types.ParamKeyTable())
	}

	return Keeper{
		storeKey:   key,
		cdc:        cdc,
		authKeeper: ak,
		bankKeeper: bk,
		paramStore: ps,
	}
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", "x/"+types.ModuleName)
}

// ActivePool returns current module's Active pool collateral balance.
func (k Keeper) ActivePool(ctx sdk.Context) sdk.Coins {
	poolAcc := k.authKeeper.GetModuleAccount(ctx, types.ActivePoolName)

	return k.bankKeeper.GetAllBalances(ctx, poolAcc.GetAddress())
}

// RedeemingPool returns current module's Redeeming pool collateral balance.
func (k Keeper) RedeemingPool(ctx sdk.Context) sdk.Coins {
	poolAcc := k.authKeeper.GetModuleAccount(ctx, types.RedeemingPoolName)

	return k.bankKeeper.GetAllBalances(ctx, poolAcc.GetAddress())
}

// ConvertCollateralsToHAL converts collateral coins to HAL coin in 1:1 relation.
func (k Keeper) ConvertCollateralsToHAL(ctx sdk.Context, colCoins sdk.Coins) (halCoin sdk.Coin, colUsedCoins sdk.Coins, retErr error) {
	halMeta, colMetas := k.HALMeta(ctx), k.CollateralMetasSet(ctx)

	halCoin = halMeta.NewZeroCoin()
	for _, colCoin := range colCoins {
		// Check if denom is supported
		colMeta, ok := colMetas[colCoin.Denom]
		if !ok {
			retErr = sdkErrors.Wrapf(types.ErrUnsupportedCollateral, "denom (%s)", colCoin.Denom)
			return
		}

		// Convert collateral -> HAL, note actually used collateral amount
		colConvertedCoin, colUsedCoin, err := colMeta.ConvertCoin2(colCoin, halMeta)
		if err != nil {
			retErr = sdkErrors.Wrapf(types.ErrInternal, "converting collateral token (%s) to HAL: %v", colCoin, err)
			return
		}
		halCoin = halCoin.Add(colConvertedCoin)
		colUsedCoins = colUsedCoins.Add(colUsedCoin)
	}

	return
}

// ConvertHALToCollaterals converts HAL coin to collateral coins in 1:1 relation iterating module's Active pool from the highest supply to the lowest.
// Returns converted HAL (equals to input if there are no leftovers)  and collaterals coins.
func (k Keeper) ConvertHALToCollaterals(ctx sdk.Context, halCoin sdk.Coin) (halUsedCoin sdk.Coin, colCoins sdk.Coins, retErr error) {
	halMeta, colMetas := k.HALMeta(ctx), k.CollateralMetasSet(ctx)

	// Check source denom
	if halCoin.Denom != halMeta.Denom {
		retErr = sdkErrors.Wrapf(types.ErrInvalidHAL, "got (%s), expected (%s)", halCoin.Denom, halMeta.Denom)
		return
	}

	// Sort active pool coins from the highest supply to the lowest normalizing amounts
	poolCoins := k.ActivePool(ctx)

	baseMeta := k.BaseMeta(ctx)
	poolCoinsNormalizedSet := make(map[string]sdk.Int)
	for _, poolCoin := range poolCoins {
		poolMeta, ok := colMetas[poolCoin.Denom]
		if !ok {
			k.Logger(ctx).Info("Collateral meta not found for ActivePool coin (skip)", "denom", poolCoin.Denom)
			continue
		}

		normalizedCoin, err := poolMeta.NormalizeCoin(poolCoin, baseMeta)
		if err != nil {
			retErr = sdkErrors.Wrapf(types.ErrInternal, "normalizing ActivePool coin (%s): %v", poolCoin, err)
			return
		}
		poolCoinsNormalizedSet[poolCoin.Denom] = normalizedCoin.Amount
	}
	sort.Slice(poolCoins, func(i, j int) bool {
		iDenom, jDenom := poolCoins[i].Denom, poolCoins[j].Denom
		iAmt, jAmt := poolCoinsNormalizedSet[poolCoins[i].Denom], poolCoinsNormalizedSet[poolCoins[j].Denom]

		if iAmt.GT(jAmt) {
			return true
		}
		if iAmt.Equal(jAmt) && iDenom > jDenom {
			return true
		}

		return false
	})

	// Fill up the desired HAL amount with the current Active pool collateral supply
	halLeftToFillCoin := halCoin
	for _, poolCoin := range poolCoins {
		poolMeta, _ := colMetas[poolCoin.Denom] // no need to check the error, since it is checked above

		// Convert collateral -> HAL to make it comparable
		poolConvertedCoin, err := poolMeta.ConvertCoin(poolCoin, halMeta)
		if err != nil {
			retErr = sdkErrors.Wrapf(types.ErrInternal, "converting pool token (%s) to HAL: %v", poolCoin, err)
			return
		}

		// Define HAL left to fill reduce amount (how much could be covered by this collateral)
		halReduceCoin := halLeftToFillCoin
		if poolConvertedCoin.IsLT(halReduceCoin) {
			halReduceCoin = poolConvertedCoin
		}

		// Convert HAL reduce amount to collateral
		colCoin, halReduceUsedCoin, err := halMeta.ConvertCoin2(halReduceCoin, poolMeta)
		if err != nil {
			retErr = sdkErrors.Wrapf(types.ErrInternal, "converting HAL reduce token (%s) to collateral denom (%s): %v", halReduceCoin, poolMeta.Denom, err)
			return
		}

		// Skip the current collateral if its supply can't cover HAL reduce amount, try the next one
		if colCoin.Amount.IsZero() {
			continue
		}

		// Apply current results
		halLeftToFillCoin = halLeftToFillCoin.Sub(halReduceUsedCoin)
		colCoins = colCoins.Add(colCoin)

		// Check if redeem amount is filled up
		if halLeftToFillCoin.IsZero() {
			break
		}
	}
	halUsedCoin = halCoin.Sub(halLeftToFillCoin)

	return
}
