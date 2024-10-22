package keeper

import (
	"encoding/binary"

	"cosmossdk.io/store/prefix"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"HalbornCTF/x/hal/types"
)

func (k Keeper) AppendTicket(ctx sdk.Context, post types.Ticket) uint64 {
	count := k.GetTicketCount(ctx)
	post.Id = count

	// Update additional info field
	metadata := map[string]string{
		"status":    "Creater",
		"timestamp": ctx.BlockTime().String(),
		"chain ID":  ctx.ChainID(),
	}

	for info, value := range metadata {
		post.AdditionalInfo += "\n" + info + ": " + value
	}

	storeAdapter := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	store := prefix.NewStore(storeAdapter, types.KeyPrefix(types.TicketKey))

	appendedValue := k.cdc.MustMarshal(&post)

	bz := make([]byte, 8)
	binary.BigEndian.PutUint64(bz, post.Id)

	store.Set(bz, appendedValue)
	k.SetTicketCounter(ctx, count+1)
	return count
}

func (k Keeper) GetTicketCount(ctx sdk.Context) uint64 {
	storeAdapter := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	store := prefix.NewStore(storeAdapter, []byte{})

	byteKey := types.KeyPrefix(types.TicketKeyCounter)
	bz := store.Get(byteKey)
	if bz == nil {
		return 0
	}
	return binary.BigEndian.Uint64(bz)
}

func (k Keeper) SetTicketCounter(ctx sdk.Context, count uint64) {
	storeAdapter := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	store := prefix.NewStore(storeAdapter, []byte{})
	byteKey := types.KeyPrefix(types.TicketKeyCounter)
	bz := make([]byte, 8)
	binary.BigEndian.PutUint64(bz, count)
	store.Set(byteKey, bz)
}
