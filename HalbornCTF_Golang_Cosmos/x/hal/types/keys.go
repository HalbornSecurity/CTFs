package types

const (
	// ModuleName defines the module name
	ModuleName = "hal"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName

	// MemStoreKey defines the in-memory store key
	MemStoreKey = "mem_hal"

	//CollateralPoolName defines module name for storing collateral coins.
	CollateralPoolName = "halborn_collateral_pool"

	//RedeemPoolName defines module name for storing collateral coins for redemption.
	RedeemPoolName = "halborn_redeem_pool"

	TicketKey = "ticket_key"

	TicketKeyCounter = "ticket_key_counter"

	TreasuryKey = "treasury_key"

	RedeemRequestKeyPrefix = "RedeemRequest-"
)

var (
	ParamsKey = []byte("p_hal")
)

func KeyPrefix(p string) []byte {
	return []byte(p)
}
