package helpers

import (
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	gaiaapp "github.com/cosmos/gaia/v7/app"
)

// GenerateAccountStrategy defines a test account generation strategy.
type GenerateAccountStrategy func(int) []sdk.AccAddress

// AddTestAddrs constructs and returns {accNum} amount of accounts with an initial balance of accAmt in random order.
func AddTestAddrs(app *gaiaapp.GaiaApp, ctx sdk.Context, accNum int, accCoins sdk.Coins) []sdk.AccAddress {
	return addTestAddrs(app, ctx, accNum, accCoins, createRandomAccounts)
}

// createRandomAccounts is a strategy used by addTestAddrs() in order to generated addresses in random order.
func createRandomAccounts(accNum int) []sdk.AccAddress {
	testAddrs := make([]sdk.AccAddress, accNum)
	for i := 0; i < accNum; i++ {
		pk := ed25519.GenPrivKey().PubKey()
		testAddrs[i] = sdk.AccAddress(pk.Address())
	}

	return testAddrs
}

func addTestAddrs(app *gaiaapp.GaiaApp, ctx sdk.Context, accNum int, accCoins sdk.Coins, strategy GenerateAccountStrategy) []sdk.AccAddress {
	testAddrs := strategy(accNum)

	for _, addr := range testAddrs {
		initAccountWithCoins(app, ctx, addr, accCoins)
	}

	return testAddrs
}

func initAccountWithCoins(app *gaiaapp.GaiaApp, ctx sdk.Context, addr sdk.AccAddress, coins sdk.Coins) {
	err := app.BankKeeper.MintCoins(ctx, minttypes.ModuleName, coins)
	if err != nil {
		panic(err)
	}

	err = app.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, addr, coins)
	if err != nil {
		panic(err)
	}
}
