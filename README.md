# rust-nodejs

Embedding Node.js in Rust.  

- Queue tasks to the Node.js event loop thread-safely.
- Interact with the Node.js runtime via [Neon](https://neon-bindings.com) API.
- Link with [prebuilt Node.js binaries](https://github.com/patr0nus/libnode/) to save compile time.
- Native modules are supported.

## Guide

1. Copy the [.cargo](https://github.com/patr0nus/rust-nodejs/tree/master/.cargo) folder in this repo to your cargo project to enable flags required to link Node.js properly. 
2. `let channel = nodejs::channel()` to get the global Node.js channel.
3. Call `channel.send` to run tasks in the Node.js event queue.
4. Inside the task, use `nodejs::neon` for interoperability between Node.js and Rust. [Neon documentation](https://docs.rs/neon/0.9.0/neon/index.html)

## Example
```rust
let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);
let channel = nodejs::channel();
channel.send(move |mut cx| {
    use nodejs::neon::{context::Context, reflect::eval, types::JsString};
    let script = cx.string("require('http').STATUS_CODES[418]");
    let whoami = eval(&mut cx, script)?;
    let whoami = whoami.downcast_or_throw::<JsString, _>(&mut cx)?;
    tx.send(whoami.value(&mut cx)).unwrap();
    Ok(())
});
let whoami = rx.recv().unwrap();
assert_eq!(whoami, "I'm a Teapot");
```
