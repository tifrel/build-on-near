use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// We need a map type and we cannot use `std`. Luckily NEAR SDK comes with
// batteries included.
use near_sdk::collections::{LazyOption, UnorderedMap};
// We require `env` to interact with the rest of the NEAR world, and of course
// the types as a "language" for these interactions.
use near_sdk::{
  env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise,
};

// We obviously need to change the storage part of our contract.
// Note how we can no longer derive `Default`, as it's not implemented for
// `UnorderedMap`. Instead we use `PanicOnDefault`, which is required by NEAR
// to signal that `Default` is not implemented for this contracts storage.
// Our other options would be to talk the `initialize` method below, and use at
// a custom implementation of `Default`.
#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct BuyMeACoffee {
  owner: AccountId,
  coffee_near_from: UnorderedMap<AccountId, Balance>,
  top_coffee_buyer: LazyOption<(AccountId, Balance)>,
}

#[near_bindgen]
impl BuyMeACoffee {
  // Because we no longer have an implementation for `Default` on our contract,
  // we need to tell NEAR how to initialize it
  #[init]
  pub fn initialize(owner: AccountId) -> Self {
    // Assert that the AccountId of the provided owner is valid, or fail
    // deployment
    assert!(env::is_valid_account_id(owner.as_bytes()));

    Self {
      owner,
      // The text inside the call to `new` is a key prefix for the onchain
      // storage key. It could just have been 0, but you should make a habit out
      // of properly prefixing your storage items. This will help whenever you
      // want to interact with raw onchain storage.
      coffee_near_from: UnorderedMap::new(b"m"),
      top_coffee_buyer: LazyOption::new(b"o", None),
    }
  }

  // This is our bread and butter-method. It needs to be payable, because that
  // is the whole point of this "Buy me a coffee" thing.
  #[payable]
  pub fn buy_coffee(&mut self) -> Promise {
    // Get call parameters
    let account = env::predecessor_account_id();
    let mut donation = env::attached_deposit();

    // Update the donation amount for the caller
    let old_donation = self.coffee_near_from.get(&account).unwrap_or(0);
    donation += old_donation;
    self.coffee_near_from.insert(&account, &donation);

    // Check if we need to update our top donor
    self.check_top_coffee_buyer(account, donation);

    // Finally, transact tokens to owner, but leave some for storage staking.
    Promise::new(self.owner.clone()).transfer(donation / 100 * 95)
  }

  fn check_top_coffee_buyer(&mut self, donor: AccountId, donation: Balance) {
    match self.top_coffee_buyer.get() {
      // Yay, we someone just bought us coffee for the first time.
      None => {
        self.top_coffee_buyer.set(&(donor, donation));
      }
      // Someone just outcompeted someone else in coffee donations for us.
      Some((_, top_donation)) if top_donation < donation => {
        self.top_coffee_buyer.set(&(donor, donation));
      }
      // In any other cases, nothing to do
      _ => {}
    }
  }

  // Get the donation amount for a specific account
  pub fn coffee_near_from(&self, account: AccountId) -> Balance {
    self.coffee_near_from.get(&account).unwrap_or(0)
  }

  // Get the account that donated most
  pub fn top_coffee_buyer(&self) -> Option<(AccountId, Balance)> {
    self.top_coffee_buyer.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::{testing_env, MockedBlockchain};
  use std::convert::TryInto;

  // Handy if you don't wish to deal with yoctoNEAR all the time
  const ONE_NEAR: near_sdk::Balance = 1_000_000_000_000_000_000_000_000;

  macro_rules! init_context {
    ($account:expr) => {
      let account = $account.to_string().try_into().unwrap();

      let context = VMContextBuilder::new()
        .predecessor_account_id(account)
        .build();

      testing_env!(context);
    };

    // Another macro pattern, as we now need to send some NEAR with our contract
    // calls
    ($account:expr, $deposit:expr) => {
      let account = $account.to_string().try_into().unwrap();

      let context = VMContextBuilder::new()
        .predecessor_account_id(account)
        .attached_deposit($deposit)
        .build();

      testing_env!(context);
    };
  }

  #[test]
  fn contract_works() {
    init_context!("tifrel.testnet");

    let mut contract = BuyMeACoffee::initialize("tifrel.testnet".into());
    assert_eq!(contract.top_coffee_buyer(), None);

    // our next contract call will be by "lovely-person.testnet", with one NEAR
    // attached to the call
    init_context!("lovely-person.testnet", 1 * ONE_NEAR);
    contract.buy_coffee();
    // We can see the donation if we query the contract by `AccountId`
    assert_eq!(
      contract.coffee_near_from("lovely-person.testnet".into()),
      1 * ONE_NEAR
    );
    // Since it's the first donation, it has taken the leaderboard
    assert_eq!(
      contract.top_coffee_buyer(),
      Some(("lovely-person.testnet".into(), 1 * ONE_NEAR))
    );

    // Let's do it again
    init_context!("another-lovely-person.testnet", 2 * ONE_NEAR);
    contract.buy_coffee();
    assert_eq!(
      contract.coffee_near_from("another-lovely-person.testnet".into()),
      2 * ONE_NEAR
    );
    assert_eq!(
      contract.top_coffee_buyer(),
      Some(("another-lovely-person.testnet".into(), 2 * ONE_NEAR))
    );
  }
}
