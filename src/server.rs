use std::{
    io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex, Barrier}, thread::{self, sleep, JoinHandle}, time::Duration
};

use crate::{client::Client, http::*, job::JobManager};

pub struct Server {
    host: String,
    port: u32,
    handle: JoinHandle<()>,
    shutdown: Arc<Mutex<bool>>,
    run_mutex: Arc<Mutex<bool>>,
}

impl Server {
    pub fn start(host: &String, port: u32, stack: Arc<Mutex<JobManager>>, fuse: Duration) -> Result<Arc<Self>, std::io::Error> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr)?;
        let shutdown = Arc::new(Mutex::new(false));
        let thread_shutdown = shutdown.clone();
        let watchdog_stack = stack.clone();

        // Create the run mutex and hold it until the server has started
        let run_mutex = Arc::new(Mutex::new(false));
        let thread_mutex = run_mutex.clone();
        let barrier = Arc::new(Barrier::new(2));
        let thread_barrier = barrier.clone();

        // Start the server thread
        let handle = thread::spawn(move || {
            let _hold = thread_mutex.lock().unwrap();
            thread_barrier.wait();
            for stream in listener.incoming() {
                if stream.is_ok() {
                    handle_connection(stream.unwrap(), stack.clone());
                }
                let lock = thread_shutdown.lock().unwrap();
                if *lock {
                    break;
                }
            }
        });

        let result = Arc::new(Self {
            host: host.clone(),
            port,
            handle,
            shutdown,
            run_mutex: run_mutex.clone(),
        });

        let watchdog_server = result.clone();

        thread::spawn(move || {
            loop {
                let mut shutdown = false;
                {
                    let check = watchdog_stack.lock().unwrap();
                    if check.is_finished() {
                        shutdown = true;
                    }
                }
                if shutdown {
                    break;
                } else {
                    sleep(Duration::new(1, 0));
                }
            }
            sleep(fuse);
            watchdog_server.stop().expect("Could not signal server stop");
        });

        barrier.wait();

        return Ok(result);
    }

    pub fn stop(&self) -> Result<(), std::io::Error> {
        let mut client = Client::new(self.host.clone(), self.port);
        {
            let mut lock = self.shutdown.lock().unwrap();
            *lock = true;
            let request = HTTPRequest::new(crate::http::HTTPMethod::GET, "server".to_string());
            client.send(request)?;
        }
        return Ok(());
    }

    pub fn wait(&self) {
        let _hold = self.run_mutex.lock().unwrap();
    }

    pub fn is_running(&self) -> bool {
        return !self.handle.is_finished();
    }
}

fn handle_connection(mut stream: TcpStream, index: Arc<Mutex<JobManager>>) {
    let buf_reader = BufReader::new(&stream);

    let request = match HTTPRequest::read(buf_reader) {
        Ok(value) => value,
        Err(code) => {
            stream.write_all(HTTPResponse::new(code).as_string().as_bytes()).unwrap();
            return;
        }
    };

    match request.method {
        HTTPMethod::GET => {
            let mut response = HTTPResponse::new(HTTPResponseCode::OK);
            if request.uri.len() > 0 {
                response = HTTPResponse::new(HTTPResponseCode::NotFound);
            } else {
                let mut payload = index.lock().unwrap();
                match (*payload).pop() {
                    Some(job) => {
                        response.content = job.to_string();
                    }
                    None => {
                        response = HTTPResponse::new(HTTPResponseCode::NoContent);
                    }
                }
            }
            stream.write_all(response.as_string().as_bytes()).unwrap();
        }
        HTTPMethod::POST => {
            let mut manager = index.lock().unwrap();
            match manager.complete(request.uri) {
                Ok(_) => { 
                    println!("{}",request.body);
                    stream.write_all(HTTPResponse::new(HTTPResponseCode::OK).as_string().as_bytes()).unwrap();
                }
                Err(_) => {
                    stream.write_all(HTTPResponse::new(HTTPResponseCode::NotFound).as_string().as_bytes()).unwrap();
                }
            }
        }
    }
}