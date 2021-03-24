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
