use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Addr};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub commodity: String,
    pub bidding_denom: String,
    pub commission: Decimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(InfoResp)]
    Info {},
    #[returns(Coin)]
    Bids {address: String},
    #[returns(HighestBidResp)]
    HighestBid {},
    #[returns(HighestBidResp)]
    Winner {},
}

#[cw_serde]
pub enum ExecMsg {
    Bid {},
    Close {},
    Retract {receiver: Option<String>},
}

#[cw_serde]
pub struct InfoResp {
    pub owner: Addr,
    pub commodity: String,
    pub bidding_denom: String,
    pub commission: Decimal,
    pub active: bool,
}

#[cw_serde]
pub struct HighestBidResp {
    pub address: String,
    pub bid: Coin,
}