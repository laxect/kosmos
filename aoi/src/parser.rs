use crate::store::Store;

const LN_XML_URI: &str = "https://www.lightnovel.us/forum.php?mod=rss&fid=173";
const LN_REPOST_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=365";
const LN_TRANSLATE_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=369";
const LN_INPUT_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=367";

trait ToMsg<T> {
    fn to_msg(self) -> Result<T, anyhow::Error>;
}

impl<T> ToMsg<T> for Result<T, surf::Exception> {
    fn to_msg(self) -> Result<T, anyhow::Error> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(anyhow::Error::msg(e.to_string())),
        }
    }
}

pub(crate) async fn fetch_rss() -> anyhow::Result<()> {
    let rss_origin = surf::get(LN_XML_URI).recv_bytes().await.to_msg()?;
    let rss_parser = rss::Channel::read_from(rss_origin.as_ref())?;
    let keyword = Store::new("keyword")?;
    let watch_list = Store::new("page")?;
    for item in rss_parser.items().into_iter() {
        let title = item.title().unwrap_or_default();
        let link = item.link().unwrap_or_default();
        // split title
        let memes = title.split(|s| s == '[' || s == ']' || s == '(' || s == ')' || s == '【' || s == '】' || s == ' ');
        let mut res = false;
        for meme in memes {
            if keyword.contains_key(meme)? {
                res = true;
                break;
            }
        }
        if res {
            watch_list.insert(&link, &title)?;
        }
    }
    Ok(())
}

pub(crate) async fn fetch_index(uri: &str) -> anyhow::Result<()> {
    let watch_list = Store::new("page")?;
    let index_html = surf::get(uri).recv_string().await.to_msg()?;

    Ok(())
}
