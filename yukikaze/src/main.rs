#![feature(box_patterns)]
mod bot;
mod postamt;

use async_std::task;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let postamt = task::spawn(async {
        let mut to_tg = postamt::kosmos_to_tg();
        if let Err(e) = to_tg.run().await {
            log::error!("kosmos to tg error: {}", e);
        }
    });
    let postamt_incoming = task::spawn(async {
        let postamt = postamt::Postamt::default();
        loop {
            if let Err(e) = postamt.incoming().await {
                log::error!("postamt - error: {}", e);
            }
            let two_sec = std::time::Duration::from_secs(2);
            task::sleep(two_sec).await;
        }
    });
    postamt.await;
    postamt_incoming.await;
    Ok(())
}
