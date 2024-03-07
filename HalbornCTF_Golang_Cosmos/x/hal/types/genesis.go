package types

import (
	"fmt"
)

// DefaultGenesisState returns GenesisState with defaults.
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		Params:        DefaultParams(),
	}
}

// Validate perform a GenesisState object validation.
func (s GenesisState) Validate() error {
	if err := s.Params.Validate(); err != nil {
		return fmt.Errorf("params: %w", err)
	}

	return nil
}
