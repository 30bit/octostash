Octostash is a cheaty way to store data on github for free. 
Given a `PERSONAL_ACCESS_TOKEN` one can create gist after gist, storing chunked data in associated files.

# Example

`Stash` struct functions as a map:

```rust
let stash = octostash::Stash::new(
    octostash::Auth::new(&env::var("MY_PERSONAL_GITHUB_ACCESS_TOKEN").unwrap()).unwrap(),
);
let id = stash.insert("Hello, octostash!").await.unwrap();
let value = stash.get(&id).await.unwrap();
assert_eq!(value, "Hello, octostash!");
stash.remove(&id).await.unwrap();
let _ = stash.get(&id).await.unwrap_err();
```

