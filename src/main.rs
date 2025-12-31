use http_server::ThreadPool;
use std::{
    error::Error,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

const BAD_REQUEST_RESPONSE: &str = "HTTP/1.1 400 Bad Request\r\n\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found";

fn main() -> Result<(), Box<dyn Error>> {
    let pool = ThreadPool::build(4)?;
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };

        pool.execute(|| {
            let _ = handle_connection(stream);
        })
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let buf_reader = BufReader::new(&stream);

    let response = handle_request(buf_reader);
    println!("Sending Response: {}", response);
    stream.write_all(response.as_bytes()).unwrap();

    Ok(())
}

fn handle_request(buf_reader: BufReader<&TcpStream>) -> String {
    let mut http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty());

    let Some(request_line) = http_request.next() else {
        return BAD_REQUEST_RESPONSE.to_string();
    };

    println!("Request-Line: {}", request_line);
    let mut request_line = request_line.split_whitespace();
    let (Some(method), Some(request_uri), Some(http_version)) = (
        request_line.next(),
        request_line.next(),
        request_line.next(),
    ) else {
        return BAD_REQUEST_RESPONSE.to_string();
    };

    if http_version != "HTTP/1.1" {
        return BAD_REQUEST_RESPONSE.to_string();
    }

    match (method, request_uri) {
        ("GET", "/") => {
            let status_line = "HTTP/1.1 200 OK";
            let contents = "Hi, my name is Josh.";
            let length = contents.len();
            let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
            response
        }
        _ => NOT_FOUND_RESPONSE.to_string(),
    }
}
