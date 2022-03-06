use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, Balance};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct DepositLogger {
  deposited: Balance,
}

#[near_bindgen]
impl DepositLogger {
  #[payable]
  pub fn deposit(&mut self, msg: String) -> U128 {
    let deposit = env::attached_deposit();
    self.deposited += deposit;

    let self_name = env::current_account_id();
    env::log(
      &format!("{}: {} (deposit: {})", self_name, msg, deposit).as_bytes(),
    );

    self.get_deposited()
  }

  pub fn get_deposited(&self) -> U128 {
    self.deposited.into()
  }
}
