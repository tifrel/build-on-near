#[derive(Default)]
pub struct Contract {
  last_number: u32,
}

impl Contract {
  pub fn execute(&mut self, x: u32) -> u32 {
    let r = self.last_number + x;
    self.last_number = x;
    r
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contract_works() {
    let mut contract = Contract::default();
    assert_eq!(contract.execute(4), 4);
    assert_eq!(contract.execute(2), 6);
  }
}
