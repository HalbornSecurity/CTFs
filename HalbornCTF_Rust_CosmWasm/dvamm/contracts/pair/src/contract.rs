use crate::error::ContractError;
use crate::state::{Config, CONFIG};

use cosmwasm_std::{
    attr, entry_point, from_json, to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Decimal256, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, Uint256, WasmMsg, WasmQuery
};

use crate::response::MsgInstantiateContractResponse;
use dvamm::asset::{Asset, AssetInfo, PairInfo};
use dvamm::factory::{FeeInfoResponse, PairType, QueryMsg as FactoryQueryMsg};

use dvamm::pair::{mint_liquidity_token_message, ConfigResponse};

use dvamm::pair::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, PoolResponse,
    QueryMsg, ReverseSimulationResponse, SimulationResponse,
};
use dvamm::querier::query_supply;
use dvamm::{token::InstantiateMsg as TokenInstantiateMsg, U256};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use protobuf::Message;
use std::convert::TryFrom;
//use std::ptr::null;
use std::vec;

// version info for migration info
const CONTRACT_NAME: &str = "dvamm-pair";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        pair_info: PairInfo {
            contract_addr: env.contract.address.clone(),
            liquidity_token: Addr::unchecked(""),
            asset_infos: msg.asset_infos,
            pair_type: PairType::Xyk {},
        },
        factory_addr: deps.api.addr_validate(msg.factory_addr.as_str())?,
        block_time_last: 0,
    };

    CONFIG.save(deps.storage, &config)?;

    // Create LP token
 
    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_json_binary(&TokenInstantiateMsg {
                name: "DVAMM LP token".to_string(),
                symbol: "dLP".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            admin: None,
            label: String::from("DVAMM LP token"),
        }
        .into(),
        id: INSTANTIATE_TOKEN_REPLY_ID,
        gas_limit: None,
        payload: Default::default(),
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))

}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.pair_info.liquidity_token != Addr::unchecked("") {
        return Err(ContractError::Unauthorized {});
    }

    let response = msg.result.into_result().unwrap().clone();

    //SubMsgResponse.data is deprecated, 
    //but SubMsgResponse.msg_responses can only be used on chains running cosmwasm2.0 or higher, 
    //otherwise it would be empty. 
    
    let message = if response.msg_responses.is_empty() {
        #[allow(deprecated)]
        response.data.as_ref().unwrap()
    } else {
        &response.msg_responses[0].value
    };

    let res: MsgInstantiateContractResponse =
    Message::parse_from_bytes(message).map_err(|_| {
        StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
    })?;
        
    config.pair_info.liquidity_token = deps.api.addr_validate(&res.contract_address)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("liquidity_token_addr", config.pair_info.liquidity_token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance,
            receiver,
        } => provide_liquidity(
            deps,
            env,
            info,
            assets,
            slippage_tolerance,
            receiver,
        ),
        ExecuteMsg::Swap {
            offer_asset,
            to,
        } => {
            if !offer_asset.is_native_token() {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(&to_addr)?)
            } else {
                None
            };

            swap(
                deps,
                env,
                info.clone(),
                info.sender,
                offer_asset,
                to_addr,
            )
        }
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone();
    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Swap {
            to,
        }) => {
            // only asset contract can execute this message
            let mut authorized: bool = false;
            let config: Config = CONFIG.load(deps.storage)?;

            for pool in config.pair_info.asset_infos {
                if let AssetInfo::Token { contract_addr, .. } = &pool {
                    if contract_addr == &info.sender {
                        authorized = true;
                    }
                }
            }

            if !authorized {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(to_addr.as_str())?)
            } else {
                None
            };

            swap(
                deps,
                env,
                info,
                Addr::unchecked(cw20_msg.sender),
                Asset {
                    info: AssetInfo::Token { contract_addr },
                    amount: cw20_msg.amount,
                },
                to_addr,
            )
        }
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => withdraw_liquidity(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
        ),
        Err(err) => Err(ContractError::Std(err)),
    }
}

/// CONTRACT - should approve contract to use the amount of token
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
    slippage_tolerance: Option<Decimal>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    for asset in assets.iter() {
        asset.assert_sent_native_token_balance(&info)?;
    }

    let config: Config = CONFIG.load(deps.storage)?;
    let mut pools: [Asset; 2] = config
        .pair_info
        .query_pools(&deps.querier, env.contract.address.clone())?;
    let deposits: [Uint128; 2] = [
        assets
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        assets
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    if deposits[0].is_zero() || deposits[1].is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter_mut().enumerate() {
        // If the pool is token contract, then we need to execute TransferFrom msg to receive funds
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: deposits[i],
                })?,
                funds: vec![],
            }));
        } else {
            // If the asset is native token, balance is already increased
            // To calculated properly we should subtract user deposit from the pool
            pool.amount = pool.amount.checked_sub(deposits[i])?;
        }
    }

    // assert slippage tolerance
    assert_slippage_tolerance(&slippage_tolerance, &deposits, &pools)?;

    let total_share = query_supply(&deps.querier, config.pair_info.liquidity_token.clone())?;
    let share = if total_share.is_zero() {
        // Initial share = collateral amount
        Uint128::new(
            (U256::from(deposits[0].u128()) * U256::from(deposits[1].u128()))
                .integer_sqrt()
                .as_u128(),
        )
    } else {
        // min(1, 2)
        // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
        // == deposit_0 * total_share / pool_0
        // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
        // == deposit_1 * total_share / pool_1
        std::cmp::min(
            deposits[0].multiply_ratio(total_share, pools[0].amount),
            deposits[1].multiply_ratio(total_share, pools[1].amount),
        )
    };

    // mint LP token for sender or receiver if set
    let receiver = receiver.unwrap_or_else(|| info.sender.to_string());
    messages.extend(mint_liquidity_token_message(
        config.pair_info.liquidity_token.clone(),
        deps.api.addr_validate(receiver.as_str())?,
        share
    )?);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "provide_liquidity"),
        attr("sender", info.sender.as_str()),
        attr("receiver", receiver.as_str()),
        attr("assets", format!("{}, {}", assets[0], assets[1])),
        attr("share", share.to_string()),
    ]))
}

pub fn withdraw_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage).unwrap();

    if info.sender != config.pair_info.liquidity_token {
        return Err(ContractError::Unauthorized {});
    }

    let (pools, total_share) = pool_info(deps.as_ref(), config.clone())?;
    let refund_assets = get_share_in_assets(&pools, amount, total_share);

    // update pool info
    let messages: Vec<CosmosMsg> = vec![
        refund_assets[0]
            .clone()
            .into_msg(sender.clone())?,
        refund_assets[1]
            .clone()
            .into_msg(sender.clone())?,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.pair_info.liquidity_token.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Burn { amount })?,
            funds: vec![],
        }),
    ];

    let attributes = vec![
        attr("action", "withdraw_liquidity"),
        attr("sender", sender.as_str()),
        attr("withdrawn_share", &amount.to_string()),
        attr(
            "refund_assets",
            format!("{}, {}", refund_assets[0], refund_assets[1]),
        ),
    ];

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes))
}

pub fn get_share_in_assets(
    pools: &[Asset; 2],
    amount: Uint128,
    total_share: Uint128,
) -> Vec<Asset> {
    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);

    pools
        .iter()
        .map(|a| Asset {
            info: a.info.clone(),
            amount: a.amount * share_ratio.to_uint_floor(),
        })
        .collect()
}
// CONTRACT - a user must do token approval
#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    offer_asset: Asset,
    to: Option<Addr>,
) -> Result<Response, ContractError> {
    offer_asset.assert_sent_native_token_balance(&info)?;

    let config: Config = CONFIG.load(deps.storage)?;

    // If the asset balance is already increased
    // To calculated properly we should subtract user deposit from the pool
    let pools: Vec<Asset> = config
        .pair_info
        .query_pools(&deps.querier, env.clone().contract.address)?
        .iter()
        .map(|p| {
            let mut p = p.clone();
            if p.info.equal(&offer_asset.info) {
                p.amount = p.amount.checked_sub(offer_asset.amount).unwrap();
            }

            p
        })
        .collect();

    let offer_pool: Asset;
    let ask_pool: Asset;

    if offer_asset.info.equal(&pools[0].info) {
        offer_pool = pools[0].clone();
        ask_pool = pools[1].clone();
    } else if offer_asset.info.equal(&pools[1].info) {
        offer_pool = pools[1].clone();
        ask_pool = pools[0].clone();
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    // Get fee info from factory
    let fee_info = get_fee_info(deps.as_ref(), config.clone())?;

    let offer_amount = offer_asset.amount;
    let (return_amount, commission_amount) = compute_swap(
        offer_pool.amount,
        ask_pool.amount,
        offer_amount,
        fee_info.total_fee_rate,
    )?;

    let return_asset = Asset {
        info: ask_pool.info.clone(),
        amount: return_amount,
    };

    let receiver = to.unwrap_or_else(|| sender.clone());
    let mut messages: Vec<CosmosMsg> =
        vec![return_asset.into_msg(receiver.clone())?];

    // Maker fee
    let mut maker_fee_amount = Uint128::new(0);
    if let Some(fee_address) = fee_info.fee_address {
        if let Some(f) = calculate_maker_fee(
            ask_pool.info.clone(),
            commission_amount,
            fee_info.maker_fee_rate,
        ) {
            messages.push(f.clone().into_msg( fee_address)?);
            maker_fee_amount = f.amount;
        }
    }

    Ok(Response::new()
        .add_messages(
            // 1. send collateral token from the contract to a user
            // 2. send inactive commission to collector
            messages,
        )
        .add_attribute("action", "swap")
        .add_attribute("sender", sender.as_str())
        .add_attribute("receiver", receiver.as_str())
        .add_attribute("offer_asset", offer_asset.info.to_string())
        .add_attribute("ask_asset", ask_pool.info.to_string())
        .add_attribute("offer_amount", offer_amount.to_string())
        .add_attribute("return_amount", return_amount.to_string())
        .add_attribute("commission_amount", commission_amount.to_string())
        .add_attribute("maker_fee_amount", maker_fee_amount.to_string()))
}

pub fn calculate_maker_fee(
    pool_info: AssetInfo,
    commission_amount: Uint128,
    maker_commission_rate: Decimal,
) -> Option<Asset> {
    let maker_fee: Uint128 = commission_amount * maker_commission_rate.to_uint_ceil();
    if maker_fee.is_zero() {
        return None;
    }

    Some(Asset {
        info: pool_info,
        amount: maker_fee,
    })
}

pub struct FeeInfo {
    pub fee_address: Option<Addr>,
    pub total_fee_rate: Decimal,
    pub maker_fee_rate: Decimal,
}

pub fn get_fee_info(deps: Deps, config: Config) -> StdResult<FeeInfo> {
    let res: FeeInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.factory_addr.to_string(),
        msg: to_json_binary(&FactoryQueryMsg::FeeInfo {
            pair_type: config.pair_info.pair_type,
        })
        .unwrap(),
    }))?;

    Ok(FeeInfo {
        fee_address: res.fee_address,
        total_fee_rate: Decimal::from_ratio(Uint128::from(res.total_fee_bps), Uint128::new(10000)),
        maker_fee_rate: Decimal::from_ratio(Uint128::from(res.maker_fee_bps), Uint128::new(10000)),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_json_binary(&query_pair_info(deps)?),
        QueryMsg::Pool {} => to_json_binary(&query_pool(deps)?),
        QueryMsg::Share { amount } => to_json_binary(&query_share(deps, amount)?),
        QueryMsg::Simulation { offer_asset } => to_json_binary(&query_simulation(deps, offer_asset)?),
        QueryMsg::ReverseSimulation { ask_asset } => {
            to_json_binary(&query_reverse_simulation(deps, ask_asset)?)
        }
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

pub fn query_pair_info(deps: Deps) -> StdResult<PairInfo> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config.pair_info)
}

pub fn query_pool(deps: Deps) -> StdResult<PoolResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let (assets, total_share) = pool_info(deps, config)?;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

pub fn query_share(deps: Deps, amount: Uint128) -> StdResult<Vec<Asset>> {
    let config: Config = CONFIG.load(deps.storage)?;
    let (pools, total_share) = pool_info(deps, config)?;
    let refund_assets = get_share_in_assets(&pools, amount, total_share);

    Ok(refund_assets)
}

pub fn query_simulation(deps: Deps, offer_asset: Asset) -> StdResult<SimulationResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let contract_addr = config.pair_info.contract_addr.clone();

    let pools: [Asset; 2] = config.pair_info.query_pools(&deps.querier, contract_addr)?;

    let offer_pool: Asset;
    let ask_pool: Asset;
    if offer_asset.info.equal(&pools[0].info) {
        offer_pool = pools[0].clone();
        ask_pool = pools[1].clone();
    } else if offer_asset.info.equal(&pools[1].info) {
        offer_pool = pools[1].clone();
        ask_pool = pools[0].clone();
    } else {
        return Err(StdError::generic_err(
            "Given offer asset doesn't belong to pairs",
        ));
    }

    // Get fee info from factory
    let fee_info = get_fee_info(deps, config)?;

    let (return_amount, commission_amount) = compute_swap(
        offer_pool.amount,
        ask_pool.amount,
        offer_asset.amount,
        fee_info.total_fee_rate,
    )?;

    Ok(SimulationResponse {
        return_amount,
        commission_amount,
    })
}

pub fn query_reverse_simulation(
    deps: Deps,
    ask_asset: Asset,
) -> StdResult<ReverseSimulationResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let contract_addr = config.pair_info.contract_addr.clone();

    let pools: [Asset; 2] = config.pair_info.query_pools(&deps.querier, contract_addr)?;

    let offer_pool: Asset;
    let ask_pool: Asset;
    if ask_asset.info.equal(&pools[0].info) {
        ask_pool = pools[0].clone();
        offer_pool = pools[1].clone();
    } else if ask_asset.info.equal(&pools[1].info) {
        ask_pool = pools[1].clone();
        offer_pool = pools[0].clone();
    } else {
        return Err(StdError::generic_err(
            "Given ask asset doesn't belong to pairs",
        ));
    }

    // Get fee info from factory
    let fee_info = get_fee_info(deps, config)?;

    let (offer_amount, commission_amount) = compute_offer_amount(
        offer_pool.amount,
        ask_pool.amount,
        ask_asset.amount,
        fee_info.total_fee_rate,
    )?;

    Ok(ReverseSimulationResponse {
        offer_amount,
        commission_amount,
    })
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        block_time_last: config.block_time_last,
    })
}

pub fn amount_of(coins: &[Coin], denom: String) -> Uint128 {
    match coins.iter().find(|x| x.denom == denom) {
        Some(coin) => coin.amount,
        None => Uint128::zero(),
    }
}

pub fn compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128,
    commission_rate: Decimal,
) -> StdResult<(Uint128, Uint128)> {
    let offer_pool: Uint256 = offer_pool.into();
    let ask_pool: Uint256 = ask_pool.into();
    let offer_amount: Uint256 = offer_amount.into();
    let commission_rate: Decimal256 = commission_rate.into();

    // offer => ask
    // ask_amount = (ask_pool - cp / (offer_pool + offer_amount))
    let cp: Uint256 = offer_pool * ask_pool;
    let return_amount: Uint256 = (Decimal256::new(ask_pool)
        - Decimal256::from_ratio(cp, offer_pool + offer_amount)).to_uint_ceil();

    // calculate commission
    let commission_amount: Uint256 = return_amount * commission_rate.to_uint_ceil();

    // commission will be absorbed to pool
    let return_amount: Uint256 = return_amount - commission_amount;
    Ok((
        Uint128::try_from(return_amount).unwrap(),
        Uint128::try_from(commission_amount).unwrap()
    ))
}

fn compute_offer_amount(
    offer_pool: Uint128,
    ask_pool: Uint128,
    ask_amount: Uint128,
    commission_rate: Decimal,
) -> StdResult<(Uint128, Uint128)> {
    // ask => offer
    // offer_amount = cp / (ask_pool - ask_amount / (1 - commission_rate)) - offer_pool
    let cp = Uint256::from(offer_pool) * Uint256::from(ask_pool);
    let one_minus_commission = Decimal256::one() - Decimal256::from(commission_rate);
    let inv_one_minus_commission: Decimal = Decimal::try_from(Decimal256::one() / one_minus_commission).unwrap();

    let offer_amount: Uint128 = Uint128::try_from(cp.multiply_ratio(
        Uint256::one(),
        Uint256::from(ask_pool.checked_sub(ask_amount * inv_one_minus_commission.to_uint_floor())?),
    )).unwrap()
    .checked_sub(offer_pool)?;

    let before_commission_deduction = ask_amount * inv_one_minus_commission.to_uint_floor();
    let commission_amount = before_commission_deduction * commission_rate.to_uint_floor();
    Ok((offer_amount, commission_amount))
}


fn assert_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Asset; 2],
) -> Result<(), ContractError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        if slippage_tolerance > Decimal256::one() {
            return Err(StdError::generic_err("slippage_tolerance cannot bigger than 1").into());
        }

        let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
        let pools: [Uint256; 2] = [pools[0].amount.into(), pools[1].amount.into()];

        // Ensure each prices are not dropped as much as slippage tolerance rate
        if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
            > Decimal256::from_ratio(pools[0], pools[1])
            || Decimal256::from_ratio(deposits[1], deposits[0]) * one_minus_slippage_tolerance
                > Decimal256::from_ratio(pools[1], pools[0])
        {
            return Err(ContractError::MaxSlippageAssertion {});
        }
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn pool_info(deps: Deps, config: Config) -> StdResult<([Asset; 2], Uint128)> {
    let contract_addr = config.pair_info.contract_addr.clone();
    let pools: [Asset; 2] = config.pair_info.query_pools(&deps.querier, contract_addr)?;
    let total_share: Uint128 = query_supply(&deps.querier, config.pair_info.liquidity_token)?;

    Ok((pools, total_share))
}
