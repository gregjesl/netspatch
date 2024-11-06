use std::{
    env, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}
};

use netspatch::{http::{HTTPMethod, HTTPRequest, HTTPResponse}, job::{create_job, Job, JobIterator, Serial}};

fn main() {
    let mut host = "localhost".to_string();
    let mut port = "7878".to_string();

    let mut args: Vec<String> = env::args().collect();
    println!("Arguements: {args:#?}");
    args.remove(0);

    let mut dimensions: Vec<usize> = Vec::new();

    while args.len() > 0 {
        match args.first().unwrap().as_str() {
            "--host" => {
                args.remove(0);
                host = args.first().unwrap().to_string();
                args.remove(0);
            }
            "--port" => {
                args.remove(0);
                port = args.first().unwrap().to_string();
                // Check for valid port
                for c in port.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid port number");
                    }
                }
                args.remove(0);
            }
            dimension => {
                for c in dimension.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid dimension {}", dimension);
                    }
                }
                dimensions.push(dimension.parse::<usize>().unwrap());
                args.remove(0);
            }
        }
    }

    if dimensions.len() == 0 {
        panic!("No dimensions provided");
    }
    assert!(dimensions.len() > 0);
    let index = Arc::new(Mutex::new(create_job(dimensions)));

    let addr = format!("{}:{}", host, port);
    println!("Address: {addr}");
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, index.clone());
    }
}

fn handle_connection(mut stream: TcpStream, index: Arc<Mutex<Job>>) {
    let buf_reader = BufReader::new(&mut stream);

    let raw: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

    match HTTPRequest::parse(raw) {
        Ok(request) => {
            match request.method {
                HTTPMethod::GET => {
                    
                    let mut response = HTTPResponse::new(netspatch::http::HTTPResponseCode::OK);
                    if request.uri.len() > 0 {
                        response = HTTPResponse::new(netspatch::http::HTTPResponseCode::NotFound);
                    } else {
                        let mut payload = index.lock().unwrap();
                        match (*payload).next() {
                            Some(job) => {
                                response.content = job.to_string();
                            }
                            None => {
                                response = HTTPResponse::new(netspatch::http::HTTPResponseCode::NoContent);
                            }
                        }
                    }
                    stream.write_all(response.as_string().as_bytes()).unwrap();
                }
                HTTPMethod::POST => {
                    stream.write_all(HTTPResponse::new(netspatch::http::HTTPResponseCode::MethodNotAllowed).as_string().as_bytes()).unwrap();
                }
            }
        }
        Err(code) => {
            stream.write_all(HTTPResponse::new(code).as_string().as_bytes()).unwrap();
        }
    }
}