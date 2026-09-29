# posta

Library-first postal protocol built on Keel.

Posta declares mailboxes, threads, letters, receipts, and their transactional
ceremonies. Keel projects that model into storage, HTTP, authority, lifecycle,
events, and estate evolution.

## Cold start

```rust
use keel::adapt::db::Sqlite;
use keel::bootstrap;
use posta::{Post, Store};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let wire = Sqlite::memory().await?;
let mut boot = bootstrap(posta::graph(), wire)?;
let token = boot.mint().await?;
let core = boot.seal(&token).await?.share();
let post = Store::new(core)
    .stage(Post {
        mailbox: "ada",
        request: "request-1",
        content: "hello",
        created: "2026-07-30T00:00:00Z",
    })
    .await?;
assert!(!post.deduplicated);
# Ok(())
# }
```

The initial protocol is the library behavior and its scenario suite. It ships
no binary and owns no runtime configuration. The crate version is the schema
version for this executable model.

## Verification

```sh
plumb configuration install
plumb guard .
```

`docs/run/verify.md` is the cold-start closure.
