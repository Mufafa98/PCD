use std::thread;

pub fn ex2() {
    let mut var: i32 = 0;
    let ptr = &mut var as *mut i32 as usize;
    let mut handlers = Vec::new();

    for _ in 0..2 {
        let handle = thread::spawn(move || {
            let var_ptr = ptr as *mut i32;
            for _ in 0..1_000_000 {
                unsafe {
                    *var_ptr += 1;
                }
            }
        });
        handlers.push(handle);
    }

    for handle in handlers {
        handle.join().unwrap();
    }

    println!("Final value: {}", var);
}
