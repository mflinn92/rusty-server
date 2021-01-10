use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use hello::ThreadPool;

static THREAD_COUNT: usize = 4;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(l) => l,
        Err(e) => panic!("Unable to bind to TCP port: {:?}", e),
    };
    let pool = match ThreadPool::new(THREAD_COUNT) {
        Ok(pool) => pool,
        Err(err) => panic!("error creating pool: {:?}", err),
    };

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        match pool.execute(|| {
            handle_connection(stream);
        }) {
            Ok(_) => continue,
            Err(e) => println!("{:?}", e),
        };
    }
    println!("Shutting Down");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("Unable to read from stream: {}", e),
    }

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(e) => panic!("Error reading file: {}: {}", filename, e),
    };
    let response = format!("{}{}", status_line, contents);

    if let Err(e) = stream.write(response.as_bytes()) {
        println!("Error writing to stream: {}", e);
    };
    if let Err(e) = stream.flush() {
        println!("Unable to flush stream: {:?}", e);
    };
}
