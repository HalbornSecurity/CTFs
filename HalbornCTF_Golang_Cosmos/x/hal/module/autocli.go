package hal

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"

	modulev1 "HalbornCTF/api/halbornctf/hal"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: modulev1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Shows the parameters of the module",
				},
				// this line is used by ignite scaffolding # autocli/query
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service:              modulev1.Msg_ServiceDesc.ServiceName,
			EnhanceCustomCommand: true, // only required if you want to use the custom command
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "UpdateParams",
					Skip:      true, // skipped because authority gated
				},
				{
					RpcMethod:      "MintHal",
					Use:            "mint-hal [collateral-amount]",
					Short:          "Send a MintHal tx",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "collateralAmount"}},
				},
				{
					RpcMethod:      "RedeemCollateral",
					Use:            "redeem-collateral [hal-amount]",
					Short:          "Send a RedeemCollateral tx",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "halAmount"}},
				},
				{
					RpcMethod:      "CreateTicket",
					Use:            "create-ticket [author] [issue]",
					Short:          "Send a CreateTicket tx",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "author"}, {ProtoField: "issue"}},
				},
				// this line is used by ignite scaffolding # autocli/tx
			},
		},
	}
}
