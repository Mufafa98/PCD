use std::{
    sync::{Arc, Barrier},
    thread::{self, JoinHandle},
    time::Duration,
};

pub fn ex1() {
    let mut handlers: Vec<JoinHandle<()>> = Vec::new();
    let barrier = Arc::new(Barrier::new(5));
    for i in 0..20 {
        let b = barrier.clone();
        let handle = thread::spawn(move || {
            println!("[Thread {:2 }]: Started", i);
            thread::sleep(Duration::from_secs(1));
            b.wait();
            println!("[Thread {:2 }]: Finished", i);
        });
        handlers.push(handle);
    }
    for handle in handlers {
        let _ = handle.join();
    }
}
