use std::collections::BTreeMap;

use mini_lust_chap6::{OrigType, TType};

#[derive(::mini_lust_macros::Message)]
pub struct Friend {
    #[mini_lust(field_id = 1, required = "true", field_type = "i32")]
    id: i32,
}

impl OrigType for Friend {}

#[derive(::mini_lust_macros::Message)]
pub struct TestUser {
    #[mini_lust(field_id = 1, required = "true", field_type = "i32")]
    id: i32,
    #[mini_lust(field_id = 2, required = "true", field_type = "string")]
    name: String,
    #[mini_lust(field_id = 3, field_type = "map(string, list(string))")]
    accounts: Option<BTreeMap<String, Vec<String>>>,
    #[mini_lust(field_id = 4, required = "true", field_type = "list(ident(Friend))")]
    friends: Vec<Friend>,
}

impl OrigType for TestUser {}

#[derive(::mini_lust_macros::Message)]
#[mini_lust(dispatch_only = true)]
pub enum MyArgs {
    MakeFriend(Friend),
    CreateTestUser(TestUser),
}

#[derive(::mini_lust_macros::Message)]
pub enum MyResult {
    #[mini_lust(field_id = 1)]
    Success(Friend),
    #[mini_lust(field_id = 2)]
    Exception(TestUser),
}

