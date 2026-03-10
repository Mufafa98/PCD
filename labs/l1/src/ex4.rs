use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
#[derive(Debug)]
struct Account {
    balance: i32,
    id: usize,
}

impl Account {
    fn new(balance: i32, id: usize) -> Self {
        Self { balance, id }
    }
}

pub fn ex4() {
    let a = Account::new(100, 1);
    let b = Account::new(100, 2);

    let a = Arc::new(Mutex::new(a));
    let b = Arc::new(Mutex::new(b));

    let a_c = a.clone();
    let b_c = b.clone();

    let t1 = thread::spawn(move || deadlock_fix(a_c, b_c, "t1"));

    let a_c = a.clone();
    let b_c = b.clone();

    let t2 = thread::spawn(move || deadlock_fix(b_c, a_c, "t2"));

    let _ = t1.join();
    let _ = t2.join();

    println!("{:?}", a);
    println!("{:?}", b);
}

fn deadlock(first: Arc<Mutex<Account>>, second: Arc<Mutex<Account>>, label: &str) {
    let mut t_f = first.lock().unwrap();
    println!("[{label}] Aquired first lock");
    thread::sleep(Duration::from_secs(1));
    let mut t_s = second.lock().unwrap();
    println!("[{label}] Aquired second lock");
    t_f.balance -= 100;
    t_s.balance += 100;
}

fn deadlock_fix(first: Arc<Mutex<Account>>, second: Arc<Mutex<Account>>, label: &str) {
    let f_id = first.lock().unwrap().id;
    let s_id = second.lock().unwrap().id;

    let (mut f, mut s) = if f_id < s_id {
        let f = first.lock().unwrap();
        println!("[{label}] Aquired first lock");
        thread::sleep(Duration::from_secs(1));
        let s = second.lock().unwrap();
        println!("[{label}] Aquired second lock");
        (f, s)
    } else {
        let s = second.lock().unwrap();
        println!("[{label}] Aquired first lock");
        thread::sleep(Duration::from_secs(1));
        let f = first.lock().unwrap();
        println!("[{label}] Aquired second lock");
        (f, s)
    };
    f.balance -= 100;
    s.balance += 100;
}
