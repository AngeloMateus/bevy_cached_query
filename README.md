# Bevy Cached Query

Simple query library for Bevy based on async tasks and Observer API loosely inspired by TanStack Query.

## Usage

Add the plugin to your Bevy app
```rust
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CachedQueryPlugin)
        .run();
```

Trigger a request
```rust
    commands.trigger(QueryBuilder::default()
        .method(Method::Get)
        .url("www.example.com")
        .build()
        .unwrap());
```
This will be added to the query cache, any subsequent calls to the same url will return the cached response.
Only the url is used to determine if a query is a duplicate, any other fields will be ignored.

Systems can then extract the reponse from the cache
```rust
    #[derive(Deserialize)]
    pub struct MyResponse {
        pub msg: String,
    }

    let response = query_extractor::<MyResponse>(
        QueryConsumable {
            url: url.to_string(),
            ..Default::default()
        },
        &mut store.cache,
    );

    if let Ok(r) = response {
        assert_eq!(r.msg, "success");
    }
```

`query_key` field can be used to avoid caching queries with the same url.

## Bevy version support

| bevy | bevy_cached_query |
| ---- | ----------------- |
| 0.14 | 0.1, main         |
