package types

import (
	"fmt"
	"time"

	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

var _ paramtypes.ParamSet = (*Params)(nil)

var (
	KeyRedeemDur                   = []byte("RedeemDur")
	DefaultRedeemDur time.Duration = 2 * 7 * 24 * time.Hour // 2 weeks
)

var (
	KeyMaxRedeemEntries = []byte("MaxRedeemEntries")
	// TODO: Determine the default value
	DefaultMaxRedeemEntries int32 = 5
)

var (
	// DefaultHalMeta defines the default HAL token metadata
	KeyHalMeta               = []byte("HalMeta")
	DefaultHalMeta TokenMeta = TokenMeta{
		Denom:       "HAL",
		Decimals:    18, // Assumed HAL has 18 decimals
		Description: "Halborn token",
	}
)

var (
	// DefaultCollateralMetas defines the default collateral token metadata
	KeyCollateralMetas               = []byte("CollateralMetas")
	DefaultCollateralMetas TokenMeta = TokenMeta{
		Denom:       "Stake",
		Decimals:    18, // Assumed default collateral has 18 decimals
		Description: "Stake token",
	}
)

// ParamKeyTable the param key table for launch module
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// NewParams creates a new Params instance
func NewParams(
	redeemDur time.Duration,
	maxRedeemEntries int32,
	collateralMetas TokenMeta,
	halMeta TokenMeta,
) Params {
	return Params{
		RedeemDur:        redeemDur,
		MaxRedeemEntries: maxRedeemEntries,
		CollateralMetas:  collateralMetas,
		HalMeta:          halMeta,
	}
}

// DefaultParams returns a default set of parameters
func DefaultParams() Params {
	return NewParams(
		DefaultRedeemDur,
		DefaultMaxRedeemEntries,
		DefaultCollateralMetas,
		DefaultHalMeta,
	)
}

// ParamSetPairs get the params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyRedeemDur, &p.RedeemDur, validateRedeemDur),
		paramtypes.NewParamSetPair(KeyMaxRedeemEntries, &p.MaxRedeemEntries, validateMaxRedeemEntries),
		paramtypes.NewParamSetPair(KeyCollateralMetas, &p.CollateralMetas, validateTokenMeta),
		paramtypes.NewParamSetPair(KeyHalMeta, &p.HalMeta, validateTokenMeta),
	}
}

// Validate validates the set of params
func (p Params) Validate() error {
	if err := validateRedeemDur(p.RedeemDur); err != nil {
		return err
	}

	if err := validateMaxRedeemEntries(p.MaxRedeemEntries); err != nil {
		return err
	}

	if err := validateTokenMeta(p.CollateralMetas); err != nil {
		return err
	}

	if err := validateTokenMeta(p.HalMeta); err != nil {
		return err
	}

	return nil
}

// validateRedeemDur validates the RedeemDur param
func validateRedeemDur(v interface{}) error {
	redeemDur, ok := v.(time.Duration)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", v)
	}

	if redeemDur <= 0 || redeemDur > 4*7*24*time.Hour {
		return fmt.Errorf("redeem duration must be positive")
	}

	return nil
}

// validateMaxRedeemEntries validates the MaxRedeemEntries param
func validateMaxRedeemEntries(v interface{}) error {
	maxRedeemEntries, ok := v.(int32)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", v)
	}

	if maxRedeemEntries < 0 {
		return fmt.Errorf("max redeem entries must be non-negative")
	}

	return nil
}

// validateTokenMeta validates a TokenMeta
func validateTokenMeta(v interface{}) error {
	_, ok := v.(TokenMeta)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", v)
	}
	// Add more validation logic for TokenMeta if needed.
	return nil
}
