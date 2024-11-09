use std::{
    env, process::exit, sync::{Arc, Mutex}, time::Duration
};

use netspatch::{job::JobManager, server::Server};

fn main() {
    let mut host = "localhost".to_string();
    let mut port = 7878_u32;
    let mut fuse = Duration::new(0, 0);

    let mut args: Vec<String> = env::args().collect();
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
                let port_str = args.first().unwrap().to_string();
                // Check for valid port
                for c in port_str.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid port number");
                    }
                }
                port = port_str.parse::<u32>().expect("Could not parse port");
                args.remove(0);
            }
            "--fuse" => {
                args.remove(0);
                let fuse_str = args.first().unwrap().to_string();
                // Check for valid port
                for c in fuse_str.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid fuse");
                    }
                }
                let fuse_len = fuse_str.parse::<u64>().expect("Could not parse fuse");
                fuse = Duration::new(fuse_len, 0);
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
        eprintln!("No dimensions provided");
        exit(1);
    }
    assert!(dimensions.len() > 0);
    let stack = Arc::new(Mutex::new(JobManager::new(&dimensions).unwrap()));

    let server = Server::start(&host, port, stack, fuse).expect("Could not start server");

    server.wait();
}
