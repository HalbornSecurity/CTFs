package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	keepertest "HalbornCTF/testutil/keeper"
	"HalbornCTF/x/hal/types"
)

func TestGetParams(t *testing.T) {
	k, ctx := keepertest.HalKeeper(t)
	params := types.DefaultParams()

	require.NoError(t, k.SetParams(ctx, params))
	require.EqualValues(t, params, k.GetParams(ctx))
}
