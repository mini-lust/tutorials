use mini_lust_chap5::{ItemService, GetUserRequest, AnonymousItemServiceGetUserResult, ApplicationResult, GetUserResponse, User, ItemServiceServer, Server};
use std::net::SocketAddr;

struct Svc;

#[async_trait::async_trait]
impl ItemService for Svc {
    async fn get_user(
        &self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> ApplicationResult<AnonymousItemServiceGetUserResult> {
        log::info!("receive a get_user request: req = {:?}, shuffle = {:?}", req, shuffle);

        let mut resp = GetUserResponse::default();
        resp.users.push(User {
            user_id: req.user_id,
            user_name: req.user_name,
            is_male: shuffle,
            extra: None,
        });
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