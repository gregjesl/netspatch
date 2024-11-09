use netspatch::{client::{Client, GetJobResult}, job::JobManager, server::Server};
use std::{sync::{Arc,Mutex}, thread::sleep, time::Duration};

fn main() {
    let host = "localhost".to_string();
    let port = 7878;

    let stack = Arc::new(
        Mutex::new(JobManager::new(&vec![2,2]).expect("Could not create stack")
    ));

    // Create the server
    print!("Attempting to start server... ");
    let server = Server::start(&host, port, stack, Duration::new(2, 0)).expect("Could not start server");
    println!("Server started");

    // Wait
    sleep(Duration::new(1, 0));

    // Create a client
    let mut client = Client::new(host, port);
    for _ in 0..5 {
        match client.query() {
            GetJobResult::JobLoaded => {
                let job = client.job.clone().unwrap();
                println!("Client: Loaded job with URI {}", job.to_uri());
                client.respond(format!("Client says \"Hello World\" in response to job {}", job.to_uri()));
            }
            GetJobResult::NoJobsLeft => {
                println!("Client: No jobs left");
                break;
            }
            GetJobResult::Error => {
                println!("Client: Error encountered");
                break;
            }
        }
        sleep(Duration::new(1, 0));
    }

    print!("Waiting for server to shut down automatically... ");

    while server.is_running() {
        sleep(Duration::new(0, 1000000));
    }
    println!("Server stopped");
}