use cosmwasm_std::{Decimal, DepsMut, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::state::{BaseInfo, BASE_INFO};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    commodity: String,
    bidding_denom: String,
    commission: Decimal,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner_addr = match owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    let base_info = BaseInfo {
        owner: owner_addr,
        commodity,
        commission,
        bidding_denom,
        active: true,
    };
    BASE_INFO.save(deps.storage, &base_info)?;
    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{BankMsg, Coin, DepsMut, MessageInfo, Response, StdError, Uint128};

    use crate::error::ContractError;
    use crate::state::{Bid, BASE_INFO, BIDS, HIGHEST_BID};

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut resp = Response::new();

        let base_info = BASE_INFO.load(deps.storage)?;
        if !base_info.active {
            return Err(ContractError::AuctionClosed {});
        }

        if info.sender == base_info.owner {
            return Err(ContractError::BiddingByOwner {});
        }

        match info
            .funds
            .iter()
            .find(|c| c.denom == base_info.bidding_denom)
        {
            Some(funds) => {
                let tax = funds.amount * base_info.commission;
                let remainder = funds.amount.checked_sub(tax).map_err(StdError::overflow)?;

                let bid = BIDS.may_load(deps.storage, info.sender.clone())?;
                let amount = bid.map_or(remainder, |b| b.amount + remainder);

                let highest_bid_amount = HIGHEST_BID
                    .may_load(deps.storage)?
                    .map(|b| b.bid.amount)
                    .unwrap_or(Uint128::zero());

                if amount <= highest_bid_amount {
                    return Err(ContractError::BidTooLow {
                        highest_bid: highest_bid_amount.to_string(),
                    });
                }
                BIDS.save(
                    deps.storage,
                    info.sender.clone(),
                    &Coin {
                        denom: funds.denom.clone(),
                        amount,
                    },
                )?;

                HIGHEST_BID.save(
                    deps.storage,
                    &Bid {
                        address: info.sender.clone(),
                        bid: Coin {
                            denom: funds.denom.clone(),
                            amount: funds.amount,
                        },
                    },
                )?;

                let bank_msg = BankMsg::Send {
                    to_address: base_info.owner.to_string(),
                    amount: vec![Coin {
                        denom: funds.denom.clone(),
                        amount: tax,
                    }],
                };

                resp = resp
                    .add_message(bank_msg)
                    .add_attribute("action", "bid")
                    .add_attribute("sender", info.sender.as_str())
                    .add_attribute("commission", tax.to_string());

                Ok(resp)
            }
            None => Err(ContractError::InvalidDenom {
                denom: base_info.bidding_denom,
            }),
        }
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut base_info = BASE_INFO.load(deps.storage)?;
        let mut resp = Response::new();

        if info.sender != base_info.owner {
            return Err(ContractError::Unauthorized {
                owner: base_info.owner.to_string(),
            });
        }

        if !base_info.active {
            return Err(ContractError::AuctionClosed {});
        }

        base_info.active = false;

        let winner = HIGHEST_BID.may_load(deps.storage)?;
        match winner {
            Some(winner) => {
                let funds = BIDS.load(deps.storage, winner.address.clone()).unwrap();

                let bank_msg = BankMsg::Send {
                    to_address: base_info.owner.to_string(),
                    amount: vec![funds.clone()],
                };

                resp = resp
                    .add_message(bank_msg)
                    .add_attribute("winner", winner.address.as_str())
                    .add_attribute("highest_bid", funds.amount);
            }
            None => {
                resp = resp.add_attribute("winner", "None");
            }
        }

        BASE_INFO.save(deps.storage, &base_info)?;

        resp = resp
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("bidding", "closed");

        Ok(resp)
    }

    pub fn retract(
        deps: DepsMut,
        info: MessageInfo,
        receiver: Option<String>,
    ) -> Result<Response, ContractError> {
        let mut resp = Response::new();
        let base_info = BASE_INFO.load(deps.storage)?;

        if base_info.active {
            return Err(ContractError::AuctionNotClosed {});
        }

        let winner = HIGHEST_BID.may_load(deps.storage)?;
        if let Some(winner) = winner {
            if info.sender == winner.address {
                return Err(ContractError::NoFundsToRetract {});
            }
        }

        let receiver_addr = receiver.unwrap_or(info.sender.to_string());
        let bids = BIDS.may_load(deps.storage, info.sender)?;
        match bids {
            Some(bid) => {
                let bank_msg = BankMsg::Send {
                    to_address: receiver_addr.clone(),
                    amount: vec![bid],
                };

                resp = resp.add_message(bank_msg)
            }
            None => {
                return Err(ContractError::NoFundsToRetract {});
            }
        }

        resp = resp
            .add_attribute("action", "retract")
            .add_attribute("sender", receiver_addr);

        Ok(resp)
    }
}

pub mod query {
    use cosmwasm_std::{Addr, Coin, Deps, StdResult, Uint128};

    use crate::msg::{HighestBidResp, InfoResp};
    use crate::state::{BASE_INFO, BIDS, HIGHEST_BID};

    pub fn info(deps: Deps) -> StdResult<InfoResp> {
        let base_info = BASE_INFO.load(deps.storage)?;

        Ok(InfoResp {
            owner: base_info.owner,
            commodity: base_info.commodity,
            bidding_denom: base_info.bidding_denom,
            commission: base_info.commission,
            active: base_info.active,
        })
    }

    pub fn bids(deps: Deps, address: String) -> StdResult<Coin> {
        let addr = Addr::unchecked(address); // Ignoring to check address format as it's not critical for the contract
        let bid = BIDS.may_load(deps.storage, addr)?;

        if let Some(bid) = bid {
            return Ok(bid);
        }

        let base_info = BASE_INFO.load(deps.storage)?;

        Ok(Coin {
            denom: base_info.bidding_denom,
            amount: Uint128::zero(),
        })
    }

    pub fn highest_bid(deps: Deps) -> StdResult<HighestBidResp> {
        let highest_bid = HIGHEST_BID.may_load(deps.storage)?;

        if let Some(highest_bid) = highest_bid {
            return Ok(HighestBidResp {
                address: highest_bid.address.to_string(),
                bid: highest_bid.bid,
            });
        }

        let base_info = BASE_INFO.load(deps.storage)?;

        Ok(HighestBidResp {
            address: "".to_string(),
            bid: Coin {
                denom: base_info.bidding_denom,
                amount: Uint128::zero(),
            },
        })
    }

    pub fn winner(deps: Deps) -> StdResult<HighestBidResp> {
        let base_info = BASE_INFO.load(deps.storage)?;
        if !base_info.active {
            return highest_bid(deps);
        }

        Ok(HighestBidResp {
            address: "".to_string(),
            bid: Coin {
                denom: base_info.bidding_denom,
                amount: Uint128::zero(),
            },
        })
    }
}
