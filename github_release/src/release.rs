use crate::target;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct User {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub x_type: String,
    pub site_admin: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Asset {
    pub url: String,
    pub browser_download_url: String,
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub label: String,
    pub state: String,
    pub content_type: String,
    pub size: u32,
    pub download_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub uploader: User,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Release {
    pub url: String,
    pub html_url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub tarball_url: String,
    pub zipball_url: String,
    pub id: u32,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub author: User,
    pub assets: Vec<Asset>,
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
