mod telegram;

async fn telegram_post(mut req: tide::Request<()>) -> anyhow::Result<&'static str> {
    let post = req.body_json().await?;
    telegram::send_message(post).await?;
    Ok("done")
}

#[async_std::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut app = tide::new();
    app.at("/telegram").post(|req| async move {
        match telegram_post(req).await {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Error: {}", e);
                "error"
            }
        }
    });
    let port = std::env::var("SORA_PORT").unwrap_or_else(|_| String::from("3000"));
    let bind_address = ["localhost:", &port].concat();
    app.listen(bind_address).await?;
    Ok(())
}
