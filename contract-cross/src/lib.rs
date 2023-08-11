use near_sdk::{
  borsh::{self, BorshDeserialize, BorshSerialize},
  env::{self, abort},
  ext_contract,
  json_types::U128,
  near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PromiseResult,
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct OtherContract {
  pub owner_id: AccountId,
  pub count: u32,
}

#[ext_contract(cross_contract)]
pub trait CrossContract {
  fn create_pool(param: bool, amount: Balance) -> U128;
}

#[near_bindgen] // Other address
impl OtherContract {
  #[init]
  pub fn init() -> Self {
    Self { owner_id: env::signer_account_id(), count: 0 }
  }
  // public
  // User > Stake -> create pool (cross contract) -> U128(1/0) -> internal();
  #[payable]
  pub fn stake(&mut self, elearning_id: AccountId, param: bool) {
    let amount = env::attached_deposit();
    cross_contract::ext(elearning_id)
      .with_static_gas(Gas(30_000_000_000))
      .create_pool(param, amount)
      .then(Self::ext(env::current_account_id()).with_static_gas(Gas(50_000_000_000)).internal(amount));
  }
  // -> stake (othercontract) -> execute create_pool (elearning_id) -> 0 ->

  #[private]
  pub fn internal(&mut self, amount: Balance) -> U128 {
    // result = 0
    let result = match env::promise_result(0) {
      PromiseResult::NotReady => abort(),
      PromiseResult::Failed => U128(0),
      PromiseResult::Successful(value) => {
        // result = 1
        if let Ok(result) = near_sdk::serde_json::from_slice::<U128>(&value) {
          result
        } else {
          U128(0)
        }
      },
    };

    if result == U128(0) {
      Promise::new(env::signer_account_id()).transfer(amount);
    } else if result == U128(1) {
      return result;
    }
    U128(0)
  }

  pub fn view_count(&self) -> u32 {
    self.count
  }
}
