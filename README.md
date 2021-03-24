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
2. Call `nodejs::event_queue()` to get the global Node.js event queue.
3. Go nuts, with `nodejs::neon` and the [EventQueue](https://docs.rs/neon/0.7.1-napi/neon/event/struct.EventQueue.html) we just got.


## Example
```rust
use nodejs::neon::{context::Context, types::JsNumber, reflect::eval};

fn main() {
    let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
    let queue = nodejs::event_queue();
    queue.send(move |mut cx| {
        let script = cx.string("require('os').freemem()");
        let free_mem = eval(&mut cx, script)?;
        let free_mem = free_mem.downcast_or_throw::<JsNumber, _>(&mut cx)?;
        tx.send(free_mem.value(&mut cx) as i64).unwrap();
        Ok(())
    });
    let free_mem = rx.recv().unwrap();
    println!("Free system memory: {}", free_mem);
}

```
