use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
  env, log, near_bindgen, AccountId, Gas, Promise, PromiseResult,
};

const INNER_GAS: Gas = Gas(500_000_000_000);
const CALLBACK_GAS: Gas = Gas(500_000_000_000);

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct CrossContractCaller {
  dispatched_calls: u32,
}

#[near_bindgen]
impl CrossContractCaller {
  #[payable]
  pub fn deposit_call(&mut self, account: AccountId, msg: String) -> Promise {
    self.dispatched_calls += 1;

    log!(
      "Dispatching call: {}.deposit({{msg: \"{}\"}})",
      account,
      msg
    );

    let deposit = env::attached_deposit();
    ext_inner::deposit(msg, account, deposit, INNER_GAS).then(
      ext_self::deposit_callback(env::current_account_id(), 0, CALLBACK_GAS),
    )
  }

  #[private]
  pub fn deposit_callback(&self) -> U128 {
    assert_eq!(
      env::promise_results_count(),
      1,
      "`deposit_callback` can only be used as callback method!"
    );

    match env::promise_result(0) {
      PromiseResult::Failed => panic!("Deposit failed!"),
      PromiseResult::NotReady => unreachable!(),
      PromiseResult::Successful(result) => {
        near_sdk::serde_json::from_slice::<U128>(&result).unwrap()
      }
    }
  }

  pub fn get_dispatched_calls(&self) -> u32 {
    self.dispatched_calls
  }
}

#[near_sdk::ext_contract(ext_inner)]
trait Inner {
  fn deposit(&mut self, msg: String) -> U128;
  fn get_deposited(&self) -> U128;
}

#[near_sdk::ext_contract(ext_self)]
trait XCCaller {
  fn deposit_callback(&self) -> U128;
}
