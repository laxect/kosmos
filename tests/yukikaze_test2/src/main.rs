use async_std::prelude::*;
use kosmos::{client::UnixClient, utils::*};
use yukikaze::{Ask, Request};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UnixClient::new("test".to_owned());
    client.regist().await?;
    let listener = client.listen().await?;
    let mut incoming = listener.incoming();
    for stream in incoming.next().await {
        let mut stream = stream?;
        let len = stream.get_len().await?;
        if len == 0 {
            return Ok(());
        }
        let Ask(obj) = stream.get_obj(len).await?;
        println!("{:?}", obj);
        let post = Request::post(obj, "test".to_owned());
        let resp = post.package()?;
        stream.write(&resp).await?;
        let exit = [0u8; 4];
        stream.write(&exit).await?;
    }
    Ok(())
}
