package hal_test

import (
	"testing"

	keepertest "HalbornCTF/testutil/keeper"
	"HalbornCTF/testutil/nullify"
	hal "HalbornCTF/x/hal/module"
	"HalbornCTF/x/hal/types"

	"github.com/stretchr/testify/require"
)

func TestGenesis(t *testing.T) {
	genesisState := types.GenesisState{
		Params: types.DefaultParams(),

		// this line is used by starport scaffolding # genesis/test/state
	}

	k, ctx := keepertest.HalKeeper(t)
	hal.InitGenesis(ctx, k, genesisState)
	got := hal.ExportGenesis(ctx, k)
	require.NotNil(t, got)

	nullify.Fill(&genesisState)
	nullify.Fill(got)

	// this line is used by starport scaffolding # genesis/test/assert
}
