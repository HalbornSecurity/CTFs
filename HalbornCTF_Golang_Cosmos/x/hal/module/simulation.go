package hal

import (
	"math/rand"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"HalbornCTF/testutil/sample"
	halsimulation "HalbornCTF/x/hal/simulation"
	"HalbornCTF/x/hal/types"
)

// avoid unused import issue
var (
	_ = halsimulation.FindAccount
	_ = rand.Rand{}
	_ = sample.AccAddress
	_ = sdk.AccAddress{}
	_ = simulation.MsgEntryKind
)

const (
	opWeightMsgMintHal = "op_weight_msg_mint_hal"
	// TODO: Determine the simulation weight value
	defaultWeightMsgMintHal int = 100

	opWeightMsgRedeemCollateral = "op_weight_msg_redeem_collateral"
	// TODO: Determine the simulation weight value
	defaultWeightMsgRedeemCollateral int = 100

	opWeightMsgCreateTicket = "op_weight_msg_create_ticket"
	// TODO: Determine the simulation weight value
	defaultWeightMsgCreateTicket int = 100

	// this line is used by starport scaffolding # simapp/module/const
)

// GenerateGenesisState creates a randomized GenState of the module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	accs := make([]string, len(simState.Accounts))
	for i, acc := range simState.Accounts {
		accs[i] = acc.Address.String()
	}
	halGenesis := types.GenesisState{
		Params: types.DefaultParams(),
		// this line is used by starport scaffolding # simapp/module/genesisState
	}
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(&halGenesis)
}

// RegisterStoreDecoder registers a decoder.
func (am AppModule) RegisterStoreDecoder(_ simtypes.StoreDecoderRegistry) {}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	operations := make([]simtypes.WeightedOperation, 0)

	var weightMsgMintHal int
	simState.AppParams.GetOrGenerate(opWeightMsgMintHal, &weightMsgMintHal, nil,
		func(_ *rand.Rand) {
			weightMsgMintHal = defaultWeightMsgMintHal
		},
	)
	operations = append(operations, simulation.NewWeightedOperation(
		weightMsgMintHal,
		halsimulation.SimulateMsgMintHal(am.accountKeeper, am.bankKeeper, am.keeper),
	))

	var weightMsgRedeemCollateral int
	simState.AppParams.GetOrGenerate(opWeightMsgRedeemCollateral, &weightMsgRedeemCollateral, nil,
		func(_ *rand.Rand) {
			weightMsgRedeemCollateral = defaultWeightMsgRedeemCollateral
		},
	)
	operations = append(operations, simulation.NewWeightedOperation(
		weightMsgRedeemCollateral,
		halsimulation.SimulateMsgRedeemCollateral(am.accountKeeper, am.bankKeeper, am.keeper),
	))

	var weightMsgCreateTicket int
	simState.AppParams.GetOrGenerate(opWeightMsgCreateTicket, &weightMsgCreateTicket, nil,
		func(_ *rand.Rand) {
			weightMsgCreateTicket = defaultWeightMsgCreateTicket
		},
	)
	operations = append(operations, simulation.NewWeightedOperation(
		weightMsgCreateTicket,
		halsimulation.SimulateMsgCreateTicket(am.accountKeeper, am.bankKeeper, am.keeper),
	))

	// this line is used by starport scaffolding # simapp/module/operation

	return operations
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (am AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsg(
			opWeightMsgMintHal,
			defaultWeightMsgMintHal,
			func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
				halsimulation.SimulateMsgMintHal(am.accountKeeper, am.bankKeeper, am.keeper)
				return nil
			},
		),
		simulation.NewWeightedProposalMsg(
			opWeightMsgRedeemCollateral,
			defaultWeightMsgRedeemCollateral,
			func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
				halsimulation.SimulateMsgRedeemCollateral(am.accountKeeper, am.bankKeeper, am.keeper)
				return nil
			},
		),
		simulation.NewWeightedProposalMsg(
			opWeightMsgCreateTicket,
			defaultWeightMsgCreateTicket,
			func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
				halsimulation.SimulateMsgCreateTicket(am.accountKeeper, am.bankKeeper, am.keeper)
				return nil
			},
		),
		// this line is used by starport scaffolding # simapp/module/OpMsg
	}
}
