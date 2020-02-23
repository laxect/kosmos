use crate::target;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct User {
    login: String,
    id: u32,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    x_type: String,
    site_admin: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Asset {
    url: String,
    browser_download_url: String,
    id: u32,
    node_id: String,
    name: String,
    label: String,
    state: String,
    content_type: String,
    size: u32,
    download_count: u32,
    created_at: String,
    updated_at: String,
    uploader: User,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Release {
    url: String,
    html_url: String,
    assets_url: String,
    upload_url: String,
    tarball_url: String,
    zipball_url: String,
    id: u32,
    node_id: String,
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    author: User,
    assets: Vec<Asset>,
}

#[derive(thiserror::Error, Debug)]
enum SurfError {
    #[error("Surf Error: {0}")]
    Error(#[from] surf::Exception),
}

async fn get_release<R: Display, U: Display>(user: U, repo: R) -> Result<Release, SurfError> {
    let uri = format!("https://api.github.com/repos/{}/{}/releases/latest", user, repo);
    let release: Release = surf::get(uri).recv_json().await?;
    Ok(release)
}

pub(crate) async fn get(target: target::Target) -> anyhow::Result<Release> {
    let user = target.get_user();
    let repo = target.get_repo();
    let release = get_release(user, repo).await?;
    Ok(release)
}
