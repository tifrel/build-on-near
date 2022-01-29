use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::{
  env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise,
};

// We need to know what the old contract state looked like
#[derive(BorshDeserialize)]
pub struct OldBuyMeACoffee {
  owner: AccountId,
  coffee_near_from: UnorderedMap<AccountId, Balance>,
  top_coffee_buyer: LazyOption<(AccountId, Balance)>,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct BuyMeACoffee {
  owner: AccountId,
  coffee_near_from: UnorderedMap<AccountId, Balance>,
  top_coffee_buyer: LazyOption<(AccountId, Balance)>,
  // This field has been added
  top_coffee_bought: LazyOption<(AccountId, Balance)>,
}

// Use standardized traits as much as you can
impl From<OldBuyMeACoffee> for BuyMeACoffee {
  fn from(old_state: OldBuyMeACoffee) -> BuyMeACoffee {
    let top_coffee_bought = LazyOption::new("o2".as_bytes(), None);
    BuyMeACoffee {
      owner: old_state.owner,
      coffee_near_from: old_state.coffee_near_from,
      top_coffee_buyer: old_state.top_coffee_buyer,
      top_coffee_bought,
    }
  }
}

#[near_bindgen]
impl BuyMeACoffee {
  #[private]
  #[init(ignore_state)]
  pub fn migrate() -> Self {
    let state = env::state_read::<OldBuyMeACoffee>()
      .expect("Couldn't deserialize prior contract state");
    state.into()
  }

  #[payable]
  pub fn buy_coffee(&mut self) -> Promise {
    // Get call parameters
    let account = env::predecessor_account_id();
    let donation_now = env::attached_deposit();

    // Update the donation amount for the caller
    let old_donation = self.coffee_near_from.get(&account).unwrap_or(0);
    let donation_acc = old_donation + donation_now;
    self.coffee_near_from.insert(&account, &donation_acc);

    // Check if we need to update our top donor
    self.check_top_coffee_buyer(account, donation_acc, donation_now);

    // Finally, transact tokens to owner, but leave some for storage staking.
    Promise::new(self.owner.clone()).transfer(donation_now / 100 * 95)
  }

  fn check_top_coffee_buyer(
    &mut self,
    donor: AccountId,
    donation_acc: Balance,
    donation_now: Balance,
  ) {
    // Check accumulated donations
    match self.top_coffee_buyer.get() {
      None => {
        self.top_coffee_buyer.set(&(donor.clone(), donation_acc));
      }
      Some((_, top_donation)) if top_donation < donation_acc => {
        self.top_coffee_buyer.set(&(donor.clone(), donation_acc));
      }
      _ => {}
    }

    // Check largest single donation
    match self.top_coffee_bought.get() {
      None => {
        self.top_coffee_bought.set(&(donor, donation_now));
      }
      Some((_, top_donation)) if top_donation < donation_now => {
        self.top_coffee_bought.set(&(donor, donation_now));
      }
      _ => {}
    }
  }

  /// Get the donation amount for a specific account
  pub fn coffee_near_from(&self, account: AccountId) -> Balance {
    self.coffee_near_from.get(&account).unwrap_or(0)
  }

  /// Get the account that cumulative donated most
  pub fn top_coffee_buyer(&self) -> Option<(AccountId, Balance)> {
    self.top_coffee_buyer.get()
  }

  /// Get the account that donated most in a single donation
  pub fn top_coffee_bought(&self) -> Option<(AccountId, Balance)> {
    self.top_coffee_bought.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::{testing_env, MockedBlockchain};
  use std::convert::TryInto;

  const ONE_NEAR: near_sdk::Balance = 1_000_000_000_000_000_000_000_000;

  macro_rules! init_context {
    // Base pattern to set caller account
    ($account:expr) => {
      let account = $account.to_string().try_into().unwrap();

      let context = VMContextBuilder::new()
        .predecessor_account_id(account)
        .build();

      testing_env!(context);
    };

    // This pattern accounts for calls to payable methods
    ($account:expr, $deposit:expr) => {
      let account = $account.to_string().try_into().unwrap();

      let context = VMContextBuilder::new()
        .predecessor_account_id(account)
        .attached_deposit($deposit)
        .build();

      testing_env!(context);
    };
  }

  // We only need this init method for testing, and since storage comes at a
  // premium, don't forget to feature-gate it for the tests
  #[near_bindgen]
  impl BuyMeACoffee {
    #[init]
    pub fn initialize(owner: AccountId) -> Self {
      assert!(env::is_valid_account_id(owner.as_bytes()));

      Self {
        owner,
        coffee_near_from: UnorderedMap::new(b"m"),
        top_coffee_buyer: LazyOption::new("o1".as_bytes(), None),
        top_coffee_bought: LazyOption::new("o2".as_bytes(), None),
      }
    }
  }

  #[test]
  fn contract_works() {
    init_context!("tifrel.testnet");

    let mut contract = BuyMeACoffee::initialize("tifrel.testnet".to_string());
    assert_eq!(contract.top_coffee_buyer(), None);

    init_context!("lovely-person.testnet", 2 * ONE_NEAR);
    contract.buy_coffee();
    assert_eq!(
      contract.coffee_near_from("lovely-person.testnet".into()),
      2 * ONE_NEAR
    );
    assert_eq!(
      contract.top_coffee_buyer(),
      Some(("lovely-person.testnet".into(), 2 * ONE_NEAR))
    );
    // Test our newly added method
    assert_eq!(
      contract.top_coffee_bought(),
      Some(("lovely-person.testnet".into(), 2 * ONE_NEAR))
    );

    init_context!("another-lovely-person.testnet", 3 * ONE_NEAR);
    contract.buy_coffee();
    assert_eq!(
      contract.coffee_near_from("another-lovely-person.testnet".into()),
      3 * ONE_NEAR
    );
    assert_eq!(
      contract.top_coffee_buyer(),
      Some(("another-lovely-person.testnet".into(), 3 * ONE_NEAR))
    );
    assert_eq!(
      contract.top_coffee_bought(),
      Some(("another-lovely-person.testnet".into(), 3 * ONE_NEAR))
    );

    // Here we see the difference, since "lovely-person.testnet" now has the
    // highest cumulative NEAR sent to us, but not the single biggest
    // transaction, that trophy still belongs to "another-lovely-person.testnet"
    init_context!("lovely-person.testnet", 2 * ONE_NEAR);
    contract.buy_coffee();
    assert_eq!(
      contract.coffee_near_from("lovely-person.testnet".into()),
      4 * ONE_NEAR
    );
    assert_eq!(
      contract.top_coffee_buyer(),
      Some(("lovely-person.testnet".into(), 4 * ONE_NEAR))
    );
    assert_eq!(
      contract.top_coffee_bought(),
      Some(("another-lovely-person.testnet".into(), 3 * ONE_NEAR))
    );
  }
}
