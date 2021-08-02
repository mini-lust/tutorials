use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct User {
    user_id: i32,
    user_name: String,
    is_male: bool,

    extra: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserRequest {
    user_id: i32,
    user_name: String,
    is_male: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserResponse {
    users: Vec<User>,
}
