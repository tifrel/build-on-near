use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise, PromiseResult};

const INNER_GAS: Gas = 500_000_000_000;
const CALLBACK_GAS: Gas = 500_000_000_000;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct CrossContractCaller {
  dispatched_calls: u32,
}

#[near_bindgen]
impl CrossContractCaller {
  #[payable]
  pub fn dispatch_call(
    &mut self,
    account: AccountId,
    method: String,
    args: String,
  ) -> Promise {
    self.dispatched_calls += 1;

    env::log(
      format!("Dispatching call: {}.{}({})", account, method, args).as_bytes(),
    );

    let deposit = env::attached_deposit();
    let xcc_promise = Promise::new(account).function_call(
      method.into_bytes(),
      args.into_bytes(),
      deposit,
      INNER_GAS,
    );
    let callback_promise = Promise::new(env::current_account_id())
      .function_call(
        "dispatch_callback".as_bytes().to_vec(),
        vec![],
        0,
        CALLBACK_GAS,
      );

    xcc_promise.then(callback_promise)
  }

  #[private]
  pub fn dispatch_callback() -> U128 {
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
