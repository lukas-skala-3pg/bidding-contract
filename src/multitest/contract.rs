use cosmwasm_std::{Addr, Coin, StdResult, Decimal};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg, QueryMsg, InfoResp, HighestBidResp};
use crate::{execute, instantiate, query};

pub struct AuctionContract(Addr);

impl AuctionContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(app: &mut App, code_id: u64, sender: &Addr, label: &str, admin: Option<String>, commodity: &str, bidding_denom: String, commission: Decimal) -> StdResult<AuctionContract> {
        
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                commodity: commodity.to_string(),
                commission,
                owner: admin.clone(),
                bidding_denom,
            },
            &[],
            label,
            admin,
        )
        .map(AuctionContract)
        .map_err(|err| err.downcast().unwrap())
    }

    pub fn bid(&self, app: &mut App, sender: &Addr, amount: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Bid {},
            &amount,
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Close {},
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn retract(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Retract {
                receiver: Some(sender.to_string()),
            },
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn query_info(&self, app: &App) -> StdResult<InfoResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Info {})
    }

    pub fn query_address(&self, app: &App, address: &Addr) -> StdResult<Coin> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Bids { address: address.to_string() })
    }

    pub fn query_highest_bid(&self, app: &App) -> StdResult<HighestBidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }
}

impl From<AuctionContract> for Addr {
    fn from(contract: AuctionContract) -> Self {
        contract.0
    }
}