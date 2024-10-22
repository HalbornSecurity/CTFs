package keeper

import (
	"fmt"

	"cosmossdk.io/core/store"
	"cosmossdk.io/log"
	math "cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"HalbornCTF/x/hal/types"
)

type (
	Keeper struct {
		cdc          codec.BinaryCodec
		storeService store.KVStoreService
		logger       log.Logger

		// the address capable of executing a MsgUpdateParams message. Typically, this
		// should be the x/gov module account.
		authority string

		accountKeeper types.AccountKeeper
		bankKeeper    types.BankKeeper
	}
)

func NewKeeper(
	cdc codec.BinaryCodec,
	storeService store.KVStoreService,
	logger log.Logger,
	authority string,

	accountKeeper types.AccountKeeper,
	bankKeeper types.BankKeeper,
) Keeper {
	if _, err := sdk.AccAddressFromBech32(authority); err != nil {
		panic(fmt.Sprintf("invalid authority address: %s", authority))
	}

	return Keeper{
		cdc:          cdc,
		storeService: storeService,
		authority:    authority,
		logger:       logger,

		accountKeeper: accountKeeper,
		bankKeeper:    bankKeeper,
	}
}

// GetAuthority returns the module's authority.
func (k Keeper) GetAuthority() string {
	return k.authority
}

// Logger returns a module-specific logger.
func (k Keeper) Logger() log.Logger {
	return k.logger.With("module", fmt.Sprintf("x/%s", types.ModuleName))
}

// SetRedeemRequest stores a redeem request in the keeper
func (k Keeper) SetRedeemRequest(ctx sdk.Context, request types.RedeemRequest) {
	store := k.storeService.OpenKVStore(ctx)
	key := []byte(types.RedeemRequestKeyPrefix + request.Account)
	bz := k.cdc.MustMarshal(&request)
	store.Set(key, bz)
}

func (k Keeper) GetAllRedeemRequests(ctx sdk.Context) []types.RedeemRequest {
	store := k.storeService.OpenKVStore(ctx)
	iterator, err := store.Iterator(nil, nil)
	if err != nil {
		panic(fmt.Sprintf("failed to create iterator: %s", err))
	}
	defer iterator.Close()

	var requests []types.RedeemRequest
	for ; iterator.Valid(); iterator.Next() {
		rawValue := iterator.Value()

		// Log the raw value for debugging
		ctx.Logger().Info("Raw store value (hex): ", fmt.Sprintf("%x", rawValue))

		var request types.RedeemRequest
		err := k.cdc.Unmarshal(rawValue, &request)
		if err != nil {
			// Log the error and continue with the next item
			ctx.Logger().Error("Failed to unmarshal redeem request: ", err)
			continue
		}

		requests = append(requests, request)
	}

	return requests
}

// DeleteRedeemRequest removes a redeem request from the keeper
func (k Keeper) DeleteRedeemRequest(ctx sdk.Context, account string) {
	store := k.storeService.OpenKVStore(ctx)
	key := []byte(types.RedeemRequestKeyPrefix + account)
	store.Delete(key)
}

// EndBlocker function to process completed redeems
func (k Keeper) EndRedeeming(ctx sdk.Context) {

	redeemRequests := k.GetAllRedeemRequests(ctx)

	for _, req := range redeemRequests {
		if ctx.BlockTime().After(req.Completiontime) {
			// Transfer the collateral tokens from RedeemPoolName to user's account
			accAddr, err := sdk.AccAddressFromBech32(req.Account)
			if err != nil {
				ctx.Logger().Error("invalid account address", "error", err)
				continue
			}
			// Get the balance of the module account after transfer
			balanceBefore := k.bankKeeper.GetBalance(ctx, accAddr, req.Collateral.Denom)

			if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.RedeemPoolName, accAddr, sdk.NewCoins(req.Collateral)); err != nil {
				ctx.Logger().Error("failed to send redeemed collateral to user", "error", err)
				continue
			}

			// Get the balance of the module account after transfer
			balanceAfter := k.bankKeeper.GetBalance(ctx, accAddr, req.Collateral.Denom)

			// Check if the balance after transfer is less than before, if not panic
			if !balanceAfter.Amount.GT(balanceBefore.Amount) {
				panic(fmt.Sprintf("User balance did not increase after the transfer. Balance before: %s, Balance after: %s", balanceBefore.Amount.String(), balanceAfter.Amount.String()))
			}

			// Remove the redeem request from the store
			k.DeleteRedeemRequest(ctx, req.Account)
		}
	}
}

func (k Keeper) InvariantCheck(ctx sdk.Context) error {
	// Retrieve params for the collateral and HAL denominations
	params := k.GetParams(ctx)
	collateralDenom := params.CollateralMetas.Denom
	halDenom := params.HalMeta.Denom

	// Check if there is any HAL supply before proceeding
	if !k.bankKeeper.HasSupply(ctx, halDenom) {
		// If there is no HAL supply, skip the check
		return nil
	}

	// Get the balance of collateral tokens in the collateral pool module
	collateralPoolBalance := k.bankKeeper.GetBalance(ctx, k.accountKeeper.GetModuleAddress(types.CollateralPoolName), collateralDenom)
	redeemPoolBalance := k.bankKeeper.GetBalance(ctx, k.accountKeeper.GetModuleAddress(types.RedeemPoolName), collateralDenom)
	poolBalance := collateralPoolBalance.Add(redeemPoolBalance)
	// Convert the collateral balance to a decimal
	collateralAmountDec := math.LegacyNewDecFromInt(poolBalance.Amount)

	// Define the exchange rate (2 collateral -> 1 HAL)
	exchangeRate := math.LegacyNewDec(2) //2

	// Divide the collateral pool balance by the exchange rate
	mintableHalAmountDec := collateralAmountDec.Mul(exchangeRate) //201 * 2 = 402

	// Retrieve the total supply of HAL tokens
	halSupply := k.bankKeeper.GetSupply(ctx, halDenom)

	// Subtract the mintable HAL amount from the total HAL supply
	result := halSupply.Amount.Sub(mintableHalAmountDec.TruncateInt())

	// Check if the result is greater than or equal to zero
	if result.IsNegative() {
		panic("invariant check failed: total HAL supply is less than the calculated amount from collateral")
	}

	return nil
}
