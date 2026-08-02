use crate::error::{conflict, engine, invalid};
use crate::{Kind, Letter};
use keel::{Op, Row, Tx, Wire, form};

pub(super) fn need(name: &str, value: &str) -> Result<String, keel::adapt::Error> {
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid(&format!("{name} must not be empty")));
    }
    if value.len() > 4096 {
        return Err(invalid(&format!("{name} is too long")));
    }
    Ok(value.to_string())
}

pub(super) async fn one<W: Wire>(
    tx: &mut Tx<'_, W>,
    unit: &str,
    field: &str,
    value: &str,
) -> Result<Option<Row>, keel::adapt::Error> {
    tx.one(&form(unit).when(field, Op::Eq, value)).await
}

pub(super) async fn mailbox<W: Wire>(
    mail: &mut Tx<'_, W>,
    tag: &str,
    created: &str,
) -> Result<i64, keel::adapt::Error> {
    if let Some(row) = one(mail, "Mailbox", "tag", tag).await? {
        return Ok(row.key());
    }
    mail.put("Mailbox", &[("tag", tag), ("created", created)])
        .await
}

pub(super) async fn thread<W: Wire>(
    bind: &mut Tx<'_, W>,
    tag: &str,
    mailbox: i64,
    created: &str,
) -> Result<i64, keel::adapt::Error> {
    if let Some(row) = one(bind, "Thread", "tag", tag).await? {
        if row.int("mailbox") == Some(mailbox) {
            return Ok(row.key());
        }
        return Err(conflict("thread belongs to another mailbox"));
    }
    let mailbox = mailbox.to_string();
    bind.put(
        "Thread",
        &[("tag", tag), ("created", created), ("mailbox", &mailbox)],
    )
    .await
}

pub(super) fn letter(row: &Row) -> Result<Letter, keel::adapt::Error> {
    let kind = match row.text("kind") {
        Some("post") => Kind::Post,
        Some("reply") => Kind::Reply,
        _ => return Err(engine("letter kind is unknown")),
    };
    Ok(Letter {
        id: row.key(),
        mailbox: row
            .int("mailbox")
            .ok_or_else(|| engine("letter mailbox is missing"))?,
        thread: row.int("thread"),
        kind,
        request: row.text("request").map(str::to_string),
        cause: row.text("cause").map(str::to_string),
        content: row
            .text("content")
            .ok_or_else(|| engine("letter content is missing"))?
            .to_string(),
        created: row
            .text("created")
            .ok_or_else(|| engine("letter creation is missing"))?
            .to_string(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Payload {
    pub kind: Kind,
    pub mailbox: i64,
    pub thread: Option<i64>,
    pub request: Option<String>,
    pub cause: Option<String>,
    pub content: String,
    pub created: String,
}

impl Payload {
    fn of(held: &Letter) -> Self {
        Self {
            kind: held.kind,
            mailbox: held.mailbox,
            thread: held.thread,
            request: held.request.clone(),
            cause: held.cause.clone(),
            content: held.content.clone(),
            created: held.created.clone(),
        }
    }
}

pub(super) fn replayed(row: &Row, want: &Payload) -> Result<Letter, keel::adapt::Error> {
    let held = letter(row)?;
    if Payload::of(&held) == *want {
        return Ok(held);
    }
    Err(conflict("replay payload changed"))
}

pub(super) async fn placed<W: Wire>(
    place: &mut Tx<'_, W>,
    id: i64,
) -> Result<Letter, keel::adapt::Error> {
    let id = id.to_string();
    let row = one(place, "Letter", "id", &id)
        .await?
        .ok_or_else(|| engine("created letter is missing"))?;
    letter(&row)
}
