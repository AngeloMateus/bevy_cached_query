# Bevy Cached Query

Simple query library for Bevy based on async tasks and Observer API very loosely inspired by TanStack Query.

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

`force_next_refetch` set to true removes the query from the cache after it has been extracted.

`ErrorTriggerEvent` is fired any time a query reponds with an error. Using the Observer API you can listen for the event and handle errors.

```rust
fn api_error_triggered(
    t: Trigger<ErrorTriggerEvent>,
    mut app_res: ResMut<ApplicationResource>,
    mut commands: Commands,
) {
    if t.event().error == 401 {
        app_res.require_authentication = true;
        commands.trigger(AuthenticateUser);
    }
}
```

Ordered queries can be used with Sequence:

```rust
commands.trigger(QuerySequence {
    key: "authenticate_user_flow".to_string(),
    tasks: vec![
        QueryBuilder::default()
            .method(Method::Post)
            .url(endpoint_from_base("api/user/auth".to_string()))
            .body(serde_json::json!({
                "username": ...,
                "password": ...
            }))
            .query_key("user_auth".to_string())
            .build()
            .unwrap(),
        QueryBuilder::default()
            .method(Method::Post)
            .url(endpoint_from_base("api/user/profile".to_string()))
            .headers(vec![("Authorization".to_string(), token)])
            .body(serde_json::json!({
                "bio": ...,
            }))
            .query_key("update_profile".to_string())
            .build()
            .unwrap(),
    ]
    .into(),
});
```

Then you can consume the requests from within a system:
```rust
let sequence = vec![
    QueryConsumable {
        url: endpoint_from_base("api/user/auth".to_string()),
        query_key: Some("user_auth".to_string()),
        ..default()
    },
    QueryConsumable {
        url: endpoint_from_base("api/user/profile".to_string()),
        query_key: Some("update_profile".to_string()),
        ..default()
    },
];

// bypassing here so we can run the system only when the store changes
if !check_completed_queries(sequence.clone(), &mut query_store.bypass_change_detection().cache) {
    return;
}

let response_user = query_extractor::<ResponseUser>(
    sequence[0].clone(),
    &mut query_store.bypass_change_detection().cache,
);
let scores = query_extractor::<Vec<TimeseriesItem>>(
    sequence[1].clone(),
    &mut query_store.bypass_change_detection().cache,
);
```

## Todo

- [x] Add staletime functionality


## Bevy version support

| bevy | bevy_cached_query |
| ---- | ----------------- |
| 0.14 | 0.2, main         |
