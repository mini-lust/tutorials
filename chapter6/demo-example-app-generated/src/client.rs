use mini_lust_chap6::SocketOrUnix;
use crate::generated::demo::{GetUserRequest, ItemServiceClientBuilder};

mod generated;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let target = SocketOrUnix::Socket("127.0.0.1:12345".parse().unwrap());
    let mut client = ItemServiceClientBuilder::new(target).build();

    let resp = client
        .GetUser(
            GetUserRequest {
                user_id: Some(1),
                user_name: Some("ihciah".to_string()),
                is_male: Some(false),
            },
            true,
        )
        .await
        .unwrap();
    log::info!("{:?}", resp);
}
