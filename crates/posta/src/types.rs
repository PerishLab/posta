#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Post,
    Reply,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Letter {
    pub id: i64,
    pub mailbox: i64,
    pub thread: Option<i64>,
    pub kind: Kind,
    pub request: Option<String>,
    pub cause: Option<String>,
    pub content: String,
    pub created: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Post<'a> {
    pub mailbox: &'a str,
    pub request: &'a str,
    pub content: &'a str,
    pub created: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Staged {
    pub letter: Letter,
    pub deduplicated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accept<'a> {
    pub request: &'a str,
    pub thread: &'a str,
    pub receipt: &'a str,
    pub created: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Accepted {
    pub receipt: i64,
    pub settled: usize,
    pub deduplicated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reply<'a> {
    pub cause: &'a str,
    pub thread: &'a str,
    pub content: &'a str,
    pub created: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replied {
    pub letter: Option<Letter>,
    pub pending: bool,
    pub deduplicated: bool,
}
