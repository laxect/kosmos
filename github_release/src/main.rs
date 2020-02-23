use async_std::task;

mod postamt;
mod release;
mod target;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let postamt_task = task::spawn(async {
        postamt::listen().await.unwrap();
    });
    postamt_task.await;
    Ok(())
}
