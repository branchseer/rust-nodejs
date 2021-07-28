# rust-nodejs

Embedding Node.js in Rust.  

- Provide a global thread-safe Node.js channel.
- Interact with the Node.js runtime via [Neon](https://neon-bindings.com) API.
- Link with [prebuilt Node.js binaries](https://github.com/patr0nus/libnode/) to save compile time.
- Native modules are supported.

## Usage

1. Add rust-nodejs to your cargo project:
```toml
[dependencies]
nodejs = "0.1.1"
```
2. `let channel = nodejs::channel()` to get the global Node.js channel.
3. Call `channel.send` to run tasks in the Node.js event queue
4. Inside the task, use `nodejs::neon` for interoperability between Node.js and Rust. [Neon documentation](https://docs.rs/neon/0.9.0/neon/index.html)
5. On macOS or Linux, add `-Clink-args=-rdynamic` to `rustflags` when building your Rust application.

## Example
```rust
use nodejs::neon::{context::Context, reflect::eval, types::JsNumber};

fn main() {
    let (tx, rx) = std::sync::mpsc::sync_channel::<i64>(0);
    let channel = nodejs::channel();
    channel.send(move |mut cx| {
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
