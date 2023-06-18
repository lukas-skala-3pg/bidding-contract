use cosmwasm_std::{coins, Addr, Coin, Decimal, Uint128};
use cw_multi_test::App;

use crate::error::ContractError;
use crate::msg::{HighestBidResp, InfoResp};

use super::contract::AuctionContract;
const ATOM: &str = "atom";
const OWNER: &str = "owner";
const BIDDER_ONE: &str = "bidder_one";
const BIDDER_TWO: &str = "bidder_two";

fn init_contract() -> (App, AuctionContract) {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(OWNER), coins(100, ATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &Addr::unchecked(BIDDER_ONE), coins(100, ATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &Addr::unchecked(BIDDER_TWO), coins(100, ATOM))
            .unwrap();
    });
    let contract_id = AuctionContract::store_code(&mut app);
    let contract = AuctionContract::instantiate(
        &mut app,
        contract_id,
        &Addr::unchecked(OWNER),
        "Auction contract",
        None,
        "Gold",
        ATOM.to_string(),
        Decimal::percent(10),
    )
    .unwrap();
    (app, contract)
}

#[test]
fn query_info_active() {
    let (app, contract) = init_contract();

    let resp = AuctionContract::query_info(&contract, &app).unwrap();

    assert_eq!(
        resp,
        InfoResp {
            owner: Addr::unchecked(OWNER),
            commodity: "Gold".to_string(),
            bidding_denom: ATOM.to_string(),
            commission: Decimal::percent(10),
            active: true,
        }
    );
}

#[test]
fn query_info_closed() {
    let (mut app, contract) = init_contract();

    AuctionContract::close(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap();

    let resp = AuctionContract::query_info(&contract, &app).unwrap();

    assert_eq!(
        resp,
        InfoResp {
            owner: Addr::unchecked(OWNER),
            commodity: "Gold".to_string(),
            bidding_denom: ATOM.to_string(),
            commission: Decimal::percent(10),
            active: false,
        }
    );
}

#[test]
fn owner_can_not_bid() {
    let (mut app, contract) = init_contract();

    let err = AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(OWNER),
        &coins(100, ATOM),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::BiddingByOwner {});
}

#[test]
fn bid_closed_auction() {
    let (mut app, contract) = init_contract();

    AuctionContract::close(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap();

    let err = AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(100, ATOM),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::AuctionClosed {});
}

#[test]
fn query_bids_by_address_no_bids() {
    let (app, contract) = init_contract();
    let resp =
        AuctionContract::query_address(&contract, &app, &Addr::unchecked(BIDDER_ONE)).unwrap();

    assert_eq!(
        resp,
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::zero(),
        }
    );
}

#[test]
fn query_bids_by_address() {
    let (mut app, contract) = init_contract();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(100, ATOM),
    )
    .unwrap();
    let resp =
        AuctionContract::query_address(&contract, &app, &Addr::unchecked(BIDDER_ONE)).unwrap();

    assert_eq!(
        resp,
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(90),
        }
    );
}

#[test]
fn query_highest_bid_no_bids() {
    let (app, contract) = init_contract();
    let resp = AuctionContract::query_highest_bid(&contract, &app).unwrap();

    assert_eq!(
        resp,
        HighestBidResp {
            address: "".to_string(),
            bid: Coin {
                denom: ATOM.to_string(),
                amount: Uint128::zero(),
            },
        }
    );
}

#[test]
fn query_highest_bid() {
    let (mut app, contract) = init_contract();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(50, ATOM),
    )
    .unwrap();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_TWO),
        &coins(100, ATOM),
    )
    .unwrap();

    let resp = AuctionContract::query_highest_bid(&contract, &app).unwrap();

    assert_eq!(
        resp,
        HighestBidResp {
            address: BIDDER_TWO.to_string(),
            bid: Coin {
                denom: ATOM.to_string(),
                amount: Uint128::new(100),
            },
        }
    );
}

#[test]
fn retract_by_winner() {
    let (mut app, contract) = init_contract();

    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(50, ATOM),
    )
    .unwrap();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_TWO),
        &coins(100, ATOM),
    )
    .unwrap();
    AuctionContract::close(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap();
    let err =
        AuctionContract::retract(&contract, &mut app, &Addr::unchecked(BIDDER_TWO)).unwrap_err();

    assert_eq!(err, ContractError::NoFundsToRetract {});
}

#[test]
fn retract_by_non_bidder() {
    let (mut app, contract) = init_contract();

    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(50, ATOM),
    )
    .unwrap();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_TWO),
        &coins(100, ATOM),
    )
    .unwrap();
    AuctionContract::close(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap();
    let err = AuctionContract::retract(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap_err();

    assert_eq!(err, ContractError::NoFundsToRetract {});
}

#[test]
fn retract_by_bidder() {
    let (mut app, contract) = init_contract();

    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_ONE),
        &coins(50, ATOM),
    )
    .unwrap();
    AuctionContract::bid(
        &contract,
        &mut app,
        &Addr::unchecked(BIDDER_TWO),
        &coins(100, ATOM),
    )
    .unwrap();
    AuctionContract::close(&contract, &mut app, &Addr::unchecked(OWNER)).unwrap();
    AuctionContract::retract(&contract, &mut app, &Addr::unchecked(BIDDER_ONE)).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(BIDDER_ONE).unwrap(),
        coins(95, ATOM)
    );
}
