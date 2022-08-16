use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Link {
    href: String,
}

#[derive(Serialize, Deserialize)]
struct LinkList {
    #[serde(rename = "self")]
    selfs: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub title: String,
    pub state: String,

    links: LinkList,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub values: Vec<PullRequest>,
}

impl PullRequest {
    pub fn link(&self) -> &str {
        return &self.links.selfs.first().unwrap().href;
    }
}
