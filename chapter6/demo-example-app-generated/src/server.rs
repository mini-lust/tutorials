use std::net::SocketAddr;

use mini_lust_chap6::{ApplicationResult, Server};

use crate::generated::demo::{
    AnonymousItemServiceGetUserResult, GetUserRequest, GetUserResponse, ItemService,
    ItemServiceServer, User,
};

mod generated;

struct Svc;

#[async_trait::async_trait]
impl ItemService for Svc {
    async fn GetUser(
        &self,
        req: Option<GetUserRequest>,
        shuffle: Option<bool>,
    ) -> ApplicationResult<AnonymousItemServiceGetUserResult> {
        log::info!(
            "receive a get_user request: req = {:?}, shuffle = {:?}",
            req,
            shuffle
        );

        let req = req.unwrap();
        let resp = GetUserResponse {
            users: vec![User {
                user_id: req.user_id.unwrap(),
                user_name: req.user_name.unwrap(),
                is_male: shuffle.unwrap(),
                extra: None,
            }]
        };
        Ok(AnonymousItemServiceGetUserResult::Success(resp))
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = Server::new(ItemServiceServer::new(Svc));
    let addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap();

    log::info!("Will serve on 127.0.0.1:12345");
    let _ = server.serve(addr).await;
}
