use std::{env, thread::sleep, time::Duration};

use netspatch::client::{Client, GetJobResult};

fn main() {
    let mut host = "localhost".to_string();
    let mut port = 7878;
    let mut id = std::process::id().to_string();
    let mut timeout = Duration::new(1, 0);
    let mut retries: u64 = 0;

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

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
            "--id" => {
                args.remove(0);
                id = args.first().unwrap().to_string();
                args.remove(0);
            }
            "--timeout" => {
                args.remove(0);
                let timeout_str = args.first().unwrap().to_string();
                for c in timeout_str.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid retry count");
                    }
                }
                let timeout_sec = timeout_str.parse::<u64>().expect("Could not parse retry count");
                timeout = Duration::new(timeout_sec, 0);
                args.remove(0);
            }
            "--retries" => {
                args.remove(0);
                let retries_str = args.first().unwrap().to_string();
                for c in retries_str.chars() {
                    if !c.is_numeric() { 
                        panic!("Invalid retry count");
                    }
                }
                retries = retries_str.parse::<u64>().expect("Could not parse retry count");
                args.remove(0);
            }
            &_ => { panic!("Unexpected arguement"); }
        }
    }

    let mut client = Client::new(host, port);
    client.with_timeout(timeout)
        .with_retries(retries);

    loop {
        match client.query() {
            GetJobResult::JobLoaded => {
                client.respond(format!("Client {id} responded to job {}", client.job.clone().unwrap().to_uri()));
            }
            GetJobResult::NoJobsLeft => {
                println!("Server reports no jobs left for client {id}. Client shutting down...");
                break;
            }
            GetJobResult::Error => {
                panic!("Error encountered");
            }
        }

        // Sleep one second to give other clients a chance
        sleep(Duration::new(1, 0));
    }
}