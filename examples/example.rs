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
