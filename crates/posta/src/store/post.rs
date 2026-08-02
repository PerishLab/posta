use super::Store;
use super::support;
use super::support::{Payload, need, one, placed, replayed};
use crate::error::{conflict, engine};
use crate::{Accept, Accepted, Error, Kind, Post, Staged};
use keel::{Op, Rank, Wire, form};

impl<W: Wire + 'static> Store<W> {
    pub async fn stage(&self, draft: Post<'_>) -> Result<Staged, Error> {
        let mailbox = need("mailbox", draft.mailbox).map_err(Error::from)?;
        let request = need("request", draft.request).map_err(Error::from)?;
        let content = need("content", draft.content).map_err(Error::from)?;
        let created = need("created", draft.created).map_err(Error::from)?;
        self.core
            .batch(async |tx| {
                let owner = support::mailbox(tx, &mailbox, &created).await?;
                if let Some(row) = one(tx, "Letter", "request", &request).await? {
                    let want = Payload {
                        kind: Kind::Post,
                        mailbox: owner,
                        thread: None,
                        request: Some(request.clone()),
                        cause: None,
                        content: content.clone(),
                        created: created.clone(),
                    };
                    return Ok(Staged {
                        letter: replayed(&row, &want)?,
                        deduplicated: true,
                    });
                }
                let owner = owner.to_string();
                let id = tx
                    .put(
                        "Letter",
                        &[
                            ("kind", "post"),
                            ("request", &request),
                            ("content", &content),
                            ("created", &created),
                            ("mailbox", &owner),
                        ],
                    )
                    .await?;
                Ok(Staged {
                    letter: placed(tx, id).await?,
                    deduplicated: false,
                })
            })
            .await
            .map_err(Error::from)
    }

    pub async fn accept(&self, draft: Accept<'_>) -> Result<Accepted, Error> {
        let request = need("request", draft.request).map_err(Error::from)?;
        let thread = need("thread", draft.thread).map_err(Error::from)?;
        let receipt = need("receipt", draft.receipt).map_err(Error::from)?;
        let created = need("created", draft.created).map_err(Error::from)?;
        self.core
            .batch(async |tx| {
                let letter = one(tx, "Letter", "request", &request)
                    .await?
                    .ok_or_else(|| conflict("request was not staged"))?;
                if letter.text("kind") != Some("post") {
                    return Err(conflict("request does not name a post"));
                }
                let owner = letter
                    .int("mailbox")
                    .ok_or_else(|| engine("staged mailbox is missing"))?;
                let thread = support::thread(tx, &thread, owner, &created).await?;
                if let Some(row) = one(tx, "Receipt", "tag", &receipt).await? {
                    if row.int("letter") == Some(letter.key())
                        && row.int("thread") == Some(thread)
                        && row.text("created") == Some(created.as_str())
                    {
                        return Ok(Accepted {
                            receipt: row.key(),
                            settled: 0,
                            deduplicated: true,
                        });
                    }
                    return Err(conflict("receipt replay payload changed"));
                }
                let letter = letter.key().to_string();
                if one(tx, "Receipt", "letter", &letter).await?.is_some() {
                    return Err(conflict("post already has another receipt"));
                }
                let thread = thread.to_string();
                let id = tx
                    .put(
                        "Receipt",
                        &[
                            ("tag", &receipt),
                            ("created", &created),
                            ("letter", &letter),
                            ("thread", &thread),
                        ],
                    )
                    .await?;
                let settled = settle(tx, &thread, owner).await?;
                Ok(Accepted {
                    receipt: id,
                    settled,
                    deduplicated: false,
                })
            })
            .await
            .map_err(Error::from)
    }
}

async fn settle<W: Wire>(
    tx: &mut keel::Tx<'_, W>,
    thread: &str,
    mailbox: i64,
) -> Result<usize, keel::adapt::Error> {
    let tag = one(tx, "Thread", "id", thread)
        .await?
        .and_then(|row| row.text("tag").map(str::to_string))
        .ok_or_else(|| engine("accepted thread is missing"))?;
    let query = form("Pending")
        .when("thread", Op::Eq, &tag)
        .order("id", Rank::Asc);
    let rows = tx.ask(&query).await?.rows().to_vec();
    for row in &rows {
        let cause = row
            .text("cause")
            .ok_or_else(|| engine("pending cause is missing"))?;
        let content = row
            .text("content")
            .ok_or_else(|| engine("pending content is missing"))?;
        let created = row
            .text("created")
            .ok_or_else(|| engine("pending creation is missing"))?;
        let mailbox = mailbox.to_string();
        tx.put(
            "Letter",
            &[
                ("kind", "reply"),
                ("cause", cause),
                ("content", content),
                ("created", created),
                ("mailbox", &mailbox),
                ("thread", thread),
            ],
        )
        .await?;
        tx.end("Pending", row.key()).await?;
    }
    Ok(rows.len())
}
