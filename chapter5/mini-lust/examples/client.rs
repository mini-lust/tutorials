use mini_lust_chap5::{GetUserRequest, SocketOrUnix, ItemServiceClientBuilder};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let target = SocketOrUnix::Socket("127.0.0.1:12345".parse().unwrap());
    let mut client = ItemServiceClientBuilder::new(target).build();

    let resp = client
        .get_user(
            GetUserRequest {
                user_id: 1,
                user_name: "ihciah".to_string(),
                is_male: false,
            },
            true,
        )
        .await
        .unwrap();
    log::info!("{:?}", resp);
}
