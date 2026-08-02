use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use keel::adapt::db::Sqlite;
use keel::{Core, bootstrap};
use posta::{Accept, Error, Kind, Post, Reply, Store};
use std::path::Path;
use std::sync::Arc;
use tower::ServiceExt;

async fn boot(path: &Path) -> Arc<Core<Sqlite>> {
    let wire = Sqlite::file(path).await.expect("wire");
    let mut boot = bootstrap(posta::graph(), wire).expect("bootstrap");
    let token = boot.mint().await.expect("mint");
    let core = boot.seal(&token).await.expect("seal");
    core.put(
        "@grant",
        &[
            ("who", "anon"),
            ("verb", "see"),
            ("unit", "*"),
            ("scope", "all"),
        ],
    )
    .await
    .expect("grant");
    core.share()
}

async fn reopen(path: &Path) -> Arc<Core<Sqlite>> {
    let wire = Sqlite::file(path).await.expect("wire");
    keel::bind(posta::graph(), wire)
        .await
        .expect("bind")
        .share()
}

#[tokio::test]
async fn postal() {
    assert_eq!(posta::VERSION, "0.1.0");
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("posta.db");
    let core = boot(&path).await;
    let store = Store::new(core.clone());

    let first = store.stage(post("hello")).await.expect("stage");
    assert!(!first.deduplicated);
    let replay = store.stage(post("hello")).await.expect("replay");
    assert!(replay.deduplicated);
    assert_eq!(replay.letter.id, first.letter.id);
    let changed = store.stage(post("changed")).await.expect_err("conflict");
    assert!(matches!(changed, Error::Conflict(_)));
    let changed = store
        .stage(Post {
            created: "2026-07-30T00:00:09Z",
            ..post("hello")
        })
        .await
        .expect_err("created conflict");
    assert!(matches!(changed, Error::Conflict(_)));

    let early = store.reply(reply("answer")).await.expect("early");
    assert!(early.pending);
    assert!(!early.deduplicated);
    let repeated = store.reply(reply("answer")).await.expect("repeat");
    assert!(repeated.pending);
    assert!(repeated.deduplicated);
    let changed = store
        .reply(Reply {
            created: "2026-07-30T00:00:09Z",
            ..reply("answer")
        })
        .await
        .expect_err("pending created conflict");
    assert!(matches!(changed, Error::Conflict(_)));
    assert_eq!(store.pending().await.expect("pending"), 1);

    drop(store);
    drop(core);

    let core = reopen(&path).await;
    let store = Store::new(core.clone());
    let accepted = store
        .accept(Accept {
            request: "request-1",
            thread: "thread-1",
            receipt: "receipt-1",
            created: "2026-07-30T00:00:02Z",
        })
        .await
        .expect("accept");
    assert_eq!(accepted.settled, 1);
    assert!(!accepted.deduplicated);
    assert_eq!(store.pending().await.expect("settled"), 0);
    let replay = store
        .accept(Accept {
            request: "request-1",
            thread: "thread-1",
            receipt: "receipt-1",
            created: "2026-07-30T00:00:02Z",
        })
        .await
        .expect("accept replay");
    assert!(replay.deduplicated);
    let changed = store
        .accept(Accept {
            request: "request-1",
            thread: "thread-1",
            receipt: "receipt-1",
            created: "2026-07-30T00:00:09Z",
        })
        .await
        .expect_err("receipt created conflict");
    assert!(matches!(changed, Error::Conflict(_)));

    let letters = store.poll("ada", 0).await.expect("poll");
    assert_eq!(letters.len(), 2);
    assert_eq!(letters[0].kind, Kind::Post);
    assert_eq!(letters[1].kind, Kind::Reply);
    let tail = store.poll("ada", letters[0].id).await.expect("tail");
    assert_eq!(tail, letters[1..]);

    let replay = store.reply(reply("answer")).await.expect("settled replay");
    assert!(replay.deduplicated);
    assert!(!replay.pending);
    assert_eq!(replay.letter.expect("letter").id, letters[1].id);
    let changed = store.reply(reply("changed")).await.expect_err("conflict");
    assert!(matches!(changed, Error::Conflict(_)));
    let changed = store
        .reply(Reply {
            created: "2026-07-30T00:00:09Z",
            ..reply("answer")
        })
        .await
        .expect_err("reply created conflict");
    assert!(matches!(changed, Error::Conflict(_)));

    projection(core).await;
}

async fn projection(core: Arc<Core<Sqlite>>) {
    let app = keel::app(core, "");
    let request = Request::builder()
        .uri("/letter")
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let rows: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(rows.as_array().expect("rows").len(), 2);

    let request = Request::builder()
        .method("GET")
        .uri("/pending")
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = serde_json::json!({
        "kind": "post",
        "request": "rogue",
        "content": "rogue",
        "created": "2026-07-30T00:00:04Z",
        "mailbox": 1
    });
    let request = Request::builder()
        .method("POST")
        .uri("/letter")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

fn post(content: &str) -> Post<'_> {
    Post {
        mailbox: "ada",
        request: "request-1",
        content,
        created: "2026-07-30T00:00:00Z",
    }
}

fn reply(content: &str) -> Reply<'_> {
    Reply {
        cause: "cause-1",
        thread: "thread-1",
        content,
        created: "2026-07-30T00:00:01Z",
    }
}
