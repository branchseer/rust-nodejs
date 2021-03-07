# rust-nodejs

Embedding Node.js in Rust.  

- Provide a global thread-safe Node.js event queue.
- Interact with the Node.js runtime via [Neon](https://neon-bindings.com) API.
- Link with [prebuilt Node.js binaries](https://github.com/patr0nus/libnode/) to save compile time.


## Usage

1. Add rust-nodejs to your cargo project:
```toml
[dependencies]
nodejs = { git = "https://github.com/patr0nus/rust-nodejs" }
```
rust-nodejs is not yet published to crates.io because it uses a [patched version](https://github.com/patr0nus/neon/tree/napi-embedding) of neon.

2. Call `nodejs::event_queue()` to get the global Node.js event queue.
3. Go nuts, with `nodejs::neon` and the [EventQueue](https://docs.rs/neon/0.7.1-napi/neon/event/struct.EventQueue.html) we just got.


## Example
```rust
use nodejs::neon::types::JsNumber;
use nodejs::NeonContextExt;

fn main() {
    let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
    let queue = nodejs::event_queue();
    queue.send(move |mut cx| {
        let free_mem = cx.run("return require('os').freemem()").ok().unwrap();
        let free_mem = free_mem.downcast::<JsNumber, _>(&mut cx).unwrap();
        tx.send(free_mem.value(&mut cx) as i64).unwrap();
        Ok(())
    });
    let free_mem = rx.recv().unwrap();
    println!("Free system memory: {}", free_mem);
}
```