// these traits allow us to convert the struct into the borsh binary format,
// which is used by NEAR and thus required for smart contracts on the protocol
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// this macro wraps our with everything necessary to deploy it to the chain.
use near_sdk::near_bindgen;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
  last_number: u32,
}

#[near_bindgen]
impl Contract {
  pub fn execute(&mut self, x: u32) -> u32 {
    let r = self.last_number + x;
    self.last_number = x;
    r
  }
}

// our familiar unit testing attribute
#[cfg(test)]
mod tests {
  // import everything we want to test
  use super::*;
  // needed for creating the blockchain context, see macro definition below
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::{testing_env, MockedBlockchain};
  use std::convert::TryInto;

  // part of writing unit tests is setting up a mock context
  macro_rules! init_context {
    ($account:expr) => {
      // first convert the `&str` to a `near_sdk::json_types::ValidAccountId`
      let account = $account.to_string().try_into().unwrap();

      // build the `near_sdk::VMContext`
      let context = VMContextBuilder::new()
        .predecessor_account_id(account)
        .build();

      // this actually initializes the context
      testing_env!(context);
    };
  }

  #[test]
  fn contract_works() {
    // initialize the testing context
    init_context!("tifrel.testnet");

    // the actual test stays as it was in the plain-rust case
    let mut contract = Contract::default();
    assert_eq!(contract.execute(4), 4);
    assert_eq!(contract.execute(2), 6);
  }
}
