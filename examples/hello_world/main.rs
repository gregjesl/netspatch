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
    let server = Server::start(&host, port, stack).expect("Could not start server");
    println!("Server started");

    // Wait
    sleep(Duration::new(1, 0));

    // Create a client
    let mut client = Client::new(host, port);
    for i in 0..5 {
        match client.query() {
            GetJobResult::JobLoaded => {
                println!("Client: Loaded job with URI {}", client.job.clone().unwrap().to_uri());
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

    print!("Attempting to shut down server... ");

    // Shutdown
    server.stop().expect("Could not stop server");
    println!("Server stopped");
}