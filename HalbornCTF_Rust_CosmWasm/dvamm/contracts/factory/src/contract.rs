use cosmwasm_std::{
    attr, entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::error::ContractError;
use crate::querier::query_pair_info;

use crate::state::{pair_key, read_pairs, Config, TmpPairInfo, CONFIG, PAIRS, TMP_PAIR_INFO};

use crate::response::MsgInstantiateContractResponse;

use dvamm::asset::{AssetInfo, PairInfo};
use dvamm::factory::{
    ConfigResponse, ExecuteMsg, FeeInfoResponse, InstantiateMsg, MigrateMsg, PairConfig, PairType,
    PairsResponse, QueryMsg,
};

use dvamm::pair::InstantiateMsg as PairInstantiateMsg;
use cw2::set_contract_version;

use protobuf::Message;

// version info for migration info
const CONTRACT_NAME: &str = "dvamm-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_PAIR_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = deps.api.addr_validate(&msg.owner)?;

    let mut config = Config {
        owner,
        token_code_id: msg.token_code_id,
        fee_address: None,
        pair_xyk_config: None,
    };

    if let Some(fee_address) = msg.fee_address {
        config.fee_address = Some(deps.api.addr_validate(fee_address.as_str())?);
    }

    if let Some(pair_xyk_config) = msg.pair_xyk_config {
        if pair_xyk_config.valid_fee_bps() {
            config.pair_xyk_config = Some(pair_xyk_config);
        } else {
            return Err(ContractError::PairConfigInvalidFeeBps {});
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

pub struct UpdateConfig {
    owner: Option<Addr>,
    token_code_id: Option<u64>,
    fee_address: Option<Addr>,
    pair_xyk_config: Option<PairConfig>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            token_code_id,
            fee_address,
            pair_xyk_config,
        } => execute_update_config(
            deps,
            env,
            info,
            UpdateConfig {
                owner,
                token_code_id,
                fee_address,
                pair_xyk_config,
            },
        ),
        ExecuteMsg::CreatePair { asset_infos } => execute_create_pair(deps, env, asset_infos),
    }
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    param: UpdateConfig,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = param.owner {
        // validate address format
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    if let Some(fee_address) = param.fee_address {
        // validate address format
        config.fee_address = Some(deps.api.addr_validate(fee_address.as_str())?);
    }

    if let Some(token_code_id) = param.token_code_id {
        config.token_code_id = token_code_id;
    }

    if let Some(pair_xyk_config) = param.pair_xyk_config {
        if pair_xyk_config.valid_fee_bps() {
            config.pair_xyk_config = Some(pair_xyk_config);
        } else {
            return Err(ContractError::PairConfigInvalidFeeBps {});
        }
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

// Anyone can execute it to create swap pair
pub fn execute_create_pair(
    deps: DepsMut,
    env: Env,
    asset_infos: [AssetInfo; 2],
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.pair_xyk_config.is_none() {
        return Err(ContractError::PairConfigNotFound {});
    }

    if PAIRS
        .may_load(deps.storage, &pair_key(&asset_infos))?
        .is_some()
    {
        return Err(ContractError::PairWasCreated {});
    }

    let pair_config = config.pair_xyk_config.unwrap();

    let pair_key = pair_key(&asset_infos);
    TMP_PAIR_INFO.save(deps.storage, &TmpPairInfo { pair_key })?;
 
    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        id: INSTANTIATE_PAIR_REPLY_ID,
        msg: WasmMsg::Instantiate {
            admin: Some(config.owner.to_string()),
            code_id: pair_config.code_id,
            msg: to_json_binary(&PairInstantiateMsg {
                asset_infos: asset_infos.clone(),
                token_code_id: config.token_code_id,
                factory_addr: env.contract.address,
            })?,
            funds: vec![],
            label: "DVAMM pair".to_string(),
        }
        .into(),
        gas_limit: None,
        payload: Default::default(),
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_submessages(sub_msg)
        .add_attributes(vec![
            attr("action", "create_pair"),
            attr("pair", format!("{}-{}", asset_infos[0], asset_infos[1])),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let tmp = TMP_PAIR_INFO.load(deps.storage)?;
    if PAIRS.may_load(deps.storage, &tmp.pair_key)?.is_some() {
        return Err(ContractError::PairWasRegistered {});
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

    let pair_contract = deps.api.addr_validate(&res.contract_address)?;

    PAIRS.save(deps.storage, &tmp.pair_key, &pair_contract)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register"),
        attr("pair_contract_addr", pair_contract),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Pair { asset_infos } => to_json_binary(&query_pair(deps, asset_infos)?),
        QueryMsg::Pairs { start_after, limit } => {
            to_json_binary(&query_pairs(deps, start_after, limit)?)
        }
        QueryMsg::FeeInfo { pair_type } => to_json_binary(&query_fee_info(deps, pair_type)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: config.owner,
        pair_xyk_config: config.pair_xyk_config,
        token_code_id: config.token_code_id,
        fee_address: config.fee_address,
    };

    Ok(resp)
}

pub fn query_pair(deps: Deps, asset_infos: [AssetInfo; 2]) -> StdResult<PairInfo> {
    let pair_addr = PAIRS.load(deps.storage, &pair_key(&asset_infos))?;
    query_pair_info(deps, &pair_addr)
}

pub fn query_pairs(
    deps: Deps,
    start_after: Option<[AssetInfo; 2]>,
    limit: Option<u32>,
) -> StdResult<PairsResponse> {
    let pairs: Vec<PairInfo> = read_pairs(deps, start_after, limit)
        .iter()
        .map(|pair_addr| query_pair_info(deps, pair_addr).unwrap())
        .collect();

    Ok(PairsResponse { pairs })
}

pub fn query_fee_info(deps: Deps, pair_type: PairType) -> StdResult<FeeInfoResponse> {
    let config = CONFIG.load(deps.storage)?;

    let pair_config = match pair_type {
        PairType::Xyk {} => config.pair_xyk_config,
    };

    if pair_config.is_none() {
        return Err(StdError::generic_err("Pair config not found"));
    }

    let pair_config = pair_config.unwrap();

    Ok(FeeInfoResponse {
        fee_address: config.fee_address,
        total_fee_bps: pair_config.total_fee_bps,
        maker_fee_bps: pair_config.maker_fee_bps,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
