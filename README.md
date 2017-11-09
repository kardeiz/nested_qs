Nested querystrings with serde
==============================

Deserializes `x-www-form-urlencoded` strings/bytes into deserializable structs and vice versa.

Similar to and inspired by [`serde_qs`](https://github.com/samscott89/serde_qs).

Defers pretty much everything except key parsing to `url::form_urlencoded` and `serde_json`.

`serde_json::Value` is used as an intermediate object between the string and your struct. However, this is not
as expensive as might be expected (you'll probably need to allocate a few strings anyway if you have any pluses
or encoded chars in your querystring). Some casual benchmarking indicates it performs well in comparison to `serde_qs`.

Using `serde_json::Value` like this does **not** mean that JSON is used in the process. No JSON strings are involved at
any point.

Use like:

```rust
let decoded: MyStruct = nested_qs::from_str(&encoded)?;
let encoded = nested_qs::to_string(&decoded)?;
```