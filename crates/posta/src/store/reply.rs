use super::Store;
use super::support::{Payload, need, one, placed, replayed};
use crate::error::conflict;
use crate::{Error, Kind, Replied, Reply};
use keel::Wire;

impl<W: Wire + 'static> Store<W> {
    pub async fn reply(&self, draft: Reply<'_>) -> Result<Replied, Error> {
        let cause = need("cause", draft.cause).map_err(Error::from)?;
        let thread = need("thread", draft.thread).map_err(Error::from)?;
        let content = need("content", draft.content).map_err(Error::from)?;
        let created = need("created", draft.created).map_err(Error::from)?;
        self.core
            .batch(async |tx| {
                let known = one(tx, "Thread", "tag", &thread).await?;
                if let Some(row) = one(tx, "Letter", "cause", &cause).await? {
                    let thread = known
                        .as_ref()
                        .map(keel::Row::key)
                        .ok_or_else(|| conflict("reply thread disappeared"))?;
                    let mailbox = row
                        .int("mailbox")
                        .ok_or_else(|| conflict("reply mailbox disappeared"))?;
                    let want = Payload {
                        kind: Kind::Reply,
                        mailbox,
                        thread: Some(thread),
                        request: None,
                        cause: Some(cause.clone()),
                        content: content.clone(),
                        created: created.clone(),
                    };
                    let letter = replayed(&row, &want)?;
                    return Ok(Replied {
                        letter: Some(letter),
                        pending: false,
                        deduplicated: true,
                    });
                }
                if let Some(row) = one(tx, "Pending", "cause", &cause).await? {
                    if row.text("thread") != Some(thread.as_str())
                        || row.text("content") != Some(content.as_str())
                        || row.text("created") != Some(created.as_str())
                    {
                        return Err(conflict("pending reply payload changed"));
                    }
                    return Ok(Replied {
                        letter: None,
                        pending: true,
                        deduplicated: true,
                    });
                }
                let Some(thread) = known else {
                    tx.put(
                        "Pending",
                        &[
                            ("cause", &cause),
                            ("thread", &thread),
                            ("content", &content),
                            ("created", &created),
                        ],
                    )
                    .await?;
                    return Ok(Replied {
                        letter: None,
                        pending: true,
                        deduplicated: false,
                    });
                };
                let mailbox = thread
                    .int("mailbox")
                    .ok_or_else(|| conflict("thread has no mailbox"))?
                    .to_string();
                let thread = thread.key().to_string();
                let id = tx
                    .put(
                        "Letter",
                        &[
                            ("kind", "reply"),
                            ("cause", &cause),
                            ("content", &content),
                            ("created", &created),
                            ("mailbox", &mailbox),
                            ("thread", &thread),
                        ],
                    )
                    .await?;
                Ok(Replied {
                    letter: Some(placed(tx, id).await?),
                    pending: false,
                    deduplicated: false,
                })
            })
            .await
            .map_err(Error::from)
    }
}
