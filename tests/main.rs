use futures_util::StreamExt as _;
use std::env;
use tokio::task::JoinSet;

fn stash() -> octostash::Stash {
    octostash::Stash::new(
        octostash::Auth::new(
            &env::var("OCTOSTASH_DEV_PERSONAL_ACCESS_TOKEN")
                .expect("OCTOSTASH_DEV_PERSONAL_ACCESS_TOKEN"),
        )
        .unwrap(),
    )
}

#[tokio::test]
async fn insert() {
    let stash = stash();
    let id = stash.insert("Hello, octostash!").await.unwrap();
    let value = stash.get(&id).await.unwrap();
    assert_eq!(value, "Hello, octostash!");
}

#[tokio::test]
async fn set() {
    let stash = stash();
    let id = stash
        .insert("Hello, octostash! Hello, octostash!")
        .await
        .unwrap();
    stash.set(&id, "Hello, octostash!").await.unwrap();
    let value = stash.get(&id).await.unwrap();
    assert_eq!(value, "Hello, octostash!");
}

#[tokio::test]
async fn remove() {
    let stash = stash();
    let id = stash.insert("Hello, octostash!").await.unwrap();
    stash.remove(&id).await.unwrap();
    let value = stash.get(&id).await.unwrap_err();
    assert_eq!(value.status(), Some(hyper::StatusCode::NOT_FOUND))
}

#[ignore]
#[tokio::test]
async fn clear() {
    let stash = stash();
    let mut ids = stash.ids();
    let mut remove_join = JoinSet::new();
    while let Some(chunk) = ids.next().await.transpose().unwrap() {
        for id in chunk {
            let stash = stash.clone();
            remove_join.spawn(async move { stash.remove(&id).await });
        }
    }
    while let Some(result) = remove_join.join_next().await {
        result.unwrap().unwrap();
    }
    assert!(stash.ids().next().await.is_none());
}
