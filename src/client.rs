use std::{io::{BufReader, Write}, net::{TcpStream, ToSocketAddrs}, time::Duration};

use crate::{http::{HTTPMethod, HTTPRequest, HTTPResponse, HTTPResponseCode}, job::Job};

pub struct Client {
    host: String,
    port: u32,
    pub job: Option<Job>,
    timeout: Duration,
    retries: u64,
}

pub enum GetJobResult {
    JobLoaded,
    NoJobsLeft,
    Error,
}

impl Client {
    pub fn new(host: String, port: u32) -> Self {
        return Self {
            host,
            port,
            job: None,
            timeout: Duration::new(1, 0),
            retries: 0
        };
    }

    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retries(&mut self, retries: u64) -> &mut Self {
        self.retries = retries;
        self
    }

    fn connect(&self) -> Result<TcpStream, std::io::Error> {
        // Build the uri
        let uri = format!("{}:{}", self.host, self.port);

        // Load the socket(s)
        let sockets = uri.to_socket_addrs()?;

        // Cache the last error
        let mut err = std::io::Error::last_os_error();

        // Check for no sockets
        let num_sockets = sockets.len();
        if num_sockets == 0 {
            eprintln!("No socket addresses found");
            return Err(err);
        }

        for _ in 0..=self.retries {
            // Loop through all socket(s)
            for socket in sockets.clone() {

                // Try to connect
                let result = TcpStream::connect_timeout(&socket, self.timeout);
                if result.is_ok() {
                    return Ok(result.unwrap());
                } else {
                    err = result.err().unwrap();   
                }
                continue;
            }
        }

        eprintln!("{num_sockets} socket(s) tried, all connections failed");
        return Err(err);
    }

    pub fn send(&mut self, request: HTTPRequest) -> Result<HTTPResponse, std::io::Error> {
        let mut stream = self.connect()?;
        
        // Send the request
        stream.write_all(request.to_string().as_bytes())?;

        // Build the reader
        let buf_reader = BufReader::new(&stream);

        // Get the response
        return match HTTPResponse::read(buf_reader) {
            Ok(value) => Ok(value),
            Err(raw) => {
                eprintln!("Error parsing HTTP response:");
                for line in raw {
                    eprintln!("  {line}");
                }
                return Err(std::io::Error::last_os_error())
            }
        };
    }

    pub fn query(&mut self) -> GetJobResult {
        // Clear the current job
        self.job = None;

        // Build the request
        let request = HTTPRequest::new(HTTPMethod::GET, "".to_string());

        // Send the request
        let response = match self.send(request) {
            Ok(value) => value,
            Err(code) => {
                eprintln!("Error during query: {}", code.to_string());
                return GetJobResult::Error
            }
        };

        // Handle the response
        match response.status {
            HTTPResponseCode::OK => {
                let job = Job::parse(&response.content);
                if job.is_err() {
                    return GetJobResult::Error;
                }
                self.job = Some(job.unwrap());
                return GetJobResult::JobLoaded;
            }
            HTTPResponseCode::NoContent => {
                self.job = None;
                return GetJobResult::NoJobsLeft;
            }
            _default => {
                eprintln!("Unexpected response code encountered");
                return GetJobResult::Error;
            }
        }
    }

    pub fn respond(&mut self, result: String) {
        if self.job.is_none() {
            panic!("Attempted to respond when no job is loaded");
        }

        // Build the request
        let job = self.job.clone().unwrap();
        let mut request = HTTPRequest::new(HTTPMethod::POST, job.to_uri());
        request.body = result;

        // Send the request
        self.send(request).expect("Error when sending job response");
        self.job = None;
    }
}