use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    thread::{self, JoinHandle},
    time::Duration,
};

fn handle_client(stream: &mut TcpStream) {
    let addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if let Ok(_) = reader.read_line(&mut line) {
        println!("[{addr}]: Received {}", line.trim());
    }

    thread::sleep(Duration::from_secs(5));

    println!("Done with client.");
}

pub fn run() {
    let listner = TcpListener::bind("127.0.0.1:8008").unwrap();

    let mut threads: Vec<JoinHandle<()>> = Vec::new();

    for stream in listner.incoming() {
        let mut stream = stream.unwrap();
        threads.push(thread::spawn(move || handle_client(&mut stream)));
    }

    for t in threads {
        let _ = t.join();
    }
}
