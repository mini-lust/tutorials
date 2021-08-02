namespace rust demo

struct A {
    1: required i32 user_id,
    2: required string user_name,
    3: required bool is_male,
    10: optional map<string, string> extra,
}

struct User {
    1: required i32 user_id,
    2: required string user_name,
    3: required bool is_male,

    10: optional map<string, string> extra,
}

struct GetUserRequest {
    1: i32 user_id,
    2: string user_name,
    3: bool is_male,
}

struct GetUserResponse {
    1: required list<User> users,
}

service ItemService {
    GetUserResponse GetUser (1: GetUserRequest req, 2: bool shuffle),
}