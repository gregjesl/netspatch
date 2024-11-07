use std::{io::{BufRead, BufReader, Read, Write}, net::{TcpStream, ToSocketAddrs}, time::Duration};

use crate::{http::{HTTPMethod, HTTPRequest, HTTPResponse}, job::Job};

pub struct Client {
    stream: TcpStream,
    job: Option<Job>,
}

impl Client {
    pub fn connect(hostname: &String, port: u32) -> Result<Client, std::io::Error> {
        let uri = format!("{hostname}:{}", port.to_string());
        let sockets = uri.to_socket_addrs()?;
        let mut err = std::io::Error::last_os_error();
        for socket in sockets {
            match TcpStream::connect_timeout(&socket, Duration::new(1, 0)) {
                Ok(stream) => return Ok(Self { 
                    stream,
                    job: None,
                }),
                Err(er) => {
                    err = er;
                    continue;
                }
            }
        }
        return Err(err);
    }

    pub fn send(&mut self, request: &HTTPRequest) -> Result<(), std::io::Error> {
        return self.stream.write_all(request.to_string().as_bytes());
    }

    pub fn get_job(&mut self) -> Option<Job> {
        let request = HTTPRequest::new(HTTPMethod::GET, "".to_string());
        let result = self.send(&request);
        if result.is_err() {
            return None;
        }
        let mut buf_reader = BufReader::new(&self.stream);

        let mut raw = Vec::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).expect("Could not read line");
            if line == "\r\n" {
                break;
            }
            line.pop();
            line.pop();
            raw.push(line);
        }
        println!("Response: {raw:#?}");
        let mut response = HTTPResponse::parse(raw)?;
        let body_len = response.expected_body_length();

        while response.content.len() < body_len {
            buf_reader.read_line(&mut response.content).expect("Could not read line");
        }

        return None;
    }

    pub fn disconnect(&mut self) -> std::io::Result<()> {
        return self.stream.shutdown(std::net::Shutdown::Both);
    }
}