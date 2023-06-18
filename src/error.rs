use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only {owner} can call it")]
    Unauthorized { owner: String },

    #[error("Auction is closed.")]
    AuctionClosed {},

    #[error("Auction isn't closed yet.")]
    AuctionNotClosed {},

    #[error("Owner can not bid.")]
    BiddingByOwner { },

    #[error("Bid with wrong coin. Must be in {denom}.")]
    InvalidDenom { denom: String },

    #[error("Bid is too low, must be higher than {highest_bid}.")]
    BidTooLow { highest_bid: String },

    #[error("No funds to retract.")]
    NoFundsToRetract {},
}