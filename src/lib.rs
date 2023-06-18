#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use error::ContractError;
use msg::InstantiateMsg;

mod contract;
pub mod error;
pub mod msg;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg.owner, msg.commodity, msg.bidding_denom, msg.commission)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Bid {} => exec::bid(deps, info),
        Close {} => exec::close(deps, info),
        Retract { receiver } => exec::retract(deps, info, receiver),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;

    match msg {
        Info {} => to_binary(&query::info(deps)?),
        Bids { address } => to_binary(&query::bids(deps, address)?),
        HighestBid {} => to_binary(&query::highest_bid(deps)?),
        Winner {} => to_binary(&query::winner(deps)?),
    }
}