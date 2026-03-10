use std::{
    sync::{Arc, Mutex},
    thread,
};

pub fn ex3() {
    let var = Arc::new(Mutex::new(0));
    let mut handlers = Vec::new();

    for _ in 0..2 {
        let v = Arc::clone(&var);

        let handle = thread::spawn(move || {
            for _ in 0..1_000_000 {
                let mut guard = v.lock().unwrap();
                *guard += 1;
            }
        });

        handlers.push(handle);
    }

    for handle in handlers {
        handle.join().unwrap();
    }

    println!("Final value: {}", *var.lock().unwrap());
}
