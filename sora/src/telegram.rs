use kosmos::client::UnixClient;
use once_cell::sync::Lazy;
use serde::Deserialize;
use yukikaze::request;

static KOSMOS: Lazy<UnixClient> = Lazy::new(|| UnixClient::new("sora"));

pub(crate) async fn send_message(post: Post) -> anyhow::Result<()> {
    log::info!("send {:?}", post);
    let req = request::post(post.msg, post.node);
    KOSMOS.send_once("yukikaze", &req).await?;
    log::info!("send success.");
    Ok(())
}

#[derive(Deserialize, Debug)]
pub(crate) struct Post {
    pub msg: String,
    pub node: String,
}
