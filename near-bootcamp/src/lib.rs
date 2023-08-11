use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct UserMetadata {
  pub name: String,
  pub user_id: AccountId,
  pub age: u8,
}

#[ext_contract(pool_contract)]
pub trait CrossCall {
  fn create_pool(param: bool) -> U128;
  fn stake(&mut self, elearning_id: AccountId, amount: Balance, param: bool);
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonUser {
  pub user_id: AccountId,
  pub user_metadata: UserMetadata,
  pub courses: Vec<CourseId>,
}

pub type CourseId = String;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CourseMetadata {
  pub course_id: CourseId,
  pub content: String,
  pub price: U128, // U128
  pub students: u32,
  pub students_completed: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct IdentityContractMetadata {
  pub spec: String,
  pub name: String,
  pub symbol: String,
  pub icon: Option<String>,
  pub base_uri: Option<String>,
  pub reference: Option<String>,
  pub reference_hash: Option<Base64VecU8>,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct ELearningContract {
  pub owner_id: AccountId,
  pub users: UnorderedSet<AccountId>,
  pub all_users: LookupMap<AccountId, JsonUser>, // Index
  pub all_courses: LookupMap<CourseId, CourseMetadata>,
}

#[near_bindgen]
impl ELearningContract {
  #[init]
  pub fn init() -> Self {
    Self::new_identity();
    Self {
      owner_id: env::signer_account_id(),
      users: UnorderedSet::new(b"user".try_to_vec().unwrap()),
      all_users: LookupMap::new(b"all_user".try_to_vec().unwrap()),
      all_courses: LookupMap::new(b"all_courses".try_to_vec().unwrap()),
    }
  }

  #[init]
  fn new_identity() -> IdentityContractMetadata {
    IdentityContractMetadata {
      spec: "elearning-1.0.0".to_string(),
      name: "ELearning Bootcamp".to_string(),
      symbol: "ELB".to_string(),
      icon: None,
      base_uri: None,
      reference: None,
      reference_hash: None,
    }
  }

  pub fn create_user(&mut self, name: String, age: u8) {
    let user = UserMetadata { name, user_id: env::signer_account_id(), age };
    let jsonuser = JsonUser { user_id: env::signer_account_id(), user_metadata: user, courses: Vec::new() };
    self.all_users.insert(&env::signer_account_id(), &jsonuser);
    self.users.insert(&env::signer_account_id());
  }

  pub fn create_course(&mut self, course_id: CourseId, content: String, price: U128) {
    let course = CourseMetadata { course_id: course_id.clone(), content, price, students: 0, students_completed: 0 };
    let mut user = self.view_user_by_id(env::signer_account_id()).unwrap();
    user.courses.push(course_id.clone());
    self.all_users.insert(&env::signer_account_id(), &user);
    self.all_courses.insert(&course_id, &course);
  }

  pub fn update_course(&mut self, course_id: CourseId, price: U128) {
    let mut course = self.get_course_by_id(course_id.clone()).unwrap();
    course.price = price;
    self.all_courses.insert(&course_id, &course);
  }

  pub fn view_user_by_id(&self, user_id: AccountId) -> Option<JsonUser> {
    if let Some(result) = self.all_users.get(&user_id) {
      Some(result)
    } else {
      None
    }
  }

  pub fn view_all_user(&self) -> Vec<JsonUser> {
    let mut users = Vec::new();
    for i in self.users.iter() {
      let result = self.view_user_by_id(i).unwrap();
      users.push(result);
    }
    users
  }

  pub fn get_course_by_id(&self, course_id: CourseId) -> Option<CourseMetadata> {
    let result = self.all_courses.get(&course_id);
    if let Some(course) = result {
      Some(course)
    } else {
      None
    }
  }

  // TODO: finish payment
  #[payable]
  pub fn payment(&mut self, course_id: CourseId) {
    let price = self.get_course_by_id(course_id).unwrap().price;
    assert_eq!(price, U128(env::attached_deposit()), "Not equal price");

    let amount: Balance = price.into();
    Promise::new(self.owner_id.clone()).transfer(amount); //
  }

  pub fn create_pool(param: bool, amount: Balance) -> U128 {
    assert_eq!(
      env::predecessor_account_id(),
      "dev-1691760216493-83287990439185".parse().unwrap(),
      "You not have author"
    );
    if param {
      U128(1)
    } else {
      U128(0)
    }
  }
}
