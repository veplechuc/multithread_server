use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

use std::time::Duration;

use webserver_lgr::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let tpool = ThreadPool::new(4);
    // to see in action the termination of the process you must add
    // to listener.incoming().. listener.incoming().take(x) wher x = numers of requests
    // this way you will tell to main only to process x numbers of requests and then shutdown

    for stream in listener.incoming().take(5) {
        // we only proces 5 requests
        let stream = stream.unwrap();
        tpool.execute(|| {
            handle_connection(stream);
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // we must consider if we are going to handle more info here or just 1024 bytes
    stream.read(&mut buffer).unwrap(); // for this example we just use unwrap... but we should check the Result type

    let get = b"GET / HTTP/1.1\r\n"; // check if the response is Ok with the Get
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let (status_line, file_name) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(10));
        ("HTTP/1.1 200 OK", "sleep.html")
    } else {
        ("HTTP/1.1 400 NOT FOUND", "404.html")
    };

    //println!("request..> {}", String::from_utf8_lossy(&buffer[..]));
    //let response = "HTTP/1.1 200 OK\r\n\r\n"; // this is a simple string response

    let file = fs::read_to_string(file_name).unwrap(); // same here

    let response = format!(
        "{}\r\nContent-Lenght: {}\r\n\r\n{}",
        status_line,
        file.len(),
        file
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
