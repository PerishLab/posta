use super::Store;
use super::support::letter;
use crate::{Error, Letter};
use keel::{Op, Rank, Wire, form};

impl<W: Wire + 'static> Store<W> {
    pub async fn poll(&self, mailbox: &str, cursor: i64) -> Result<Vec<Letter>, Error> {
        let mailbox = mailbox.trim();
        if mailbox.is_empty() {
            return Err(crate::Error::Invalid(
                "mailbox must not be empty".to_string(),
            ));
        }
        let Some(mailbox) = self
            .core
            .one(&form("Mailbox").when("tag", Op::Eq, mailbox))
            .await
            .map_err(Error::from)?
        else {
            return Ok(Vec::new());
        };
        let mailbox = mailbox.key().to_string();
        let mut query = form("Letter")
            .when("mailbox", Op::Eq, &mailbox)
            .order("id", Rank::Asc);
        if cursor > 0 {
            query = query.past(cursor);
        }
        self.core
            .ask(&query)
            .await
            .map_err(Error::from)?
            .rows()
            .iter()
            .map(letter)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    pub async fn pending(&self) -> Result<usize, Error> {
        let pack = self
            .core
            .query("from Pending count")
            .await
            .map_err(Error::from)?;
        Ok(pack.count().unwrap_or_default())
    }
}
