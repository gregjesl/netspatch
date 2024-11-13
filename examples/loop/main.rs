use netspatch::{client::Client, job::JobManager, server::Server};
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

    // Create the client
    let mut client = Client::new(host, port);

    // Loop through the jobs
    while client.query().success() {
        client.respond(format!("Client says \"Hello World\" in response to job {}", client.job.clone().unwrap().to_uri()));
    }

    print!("Waiting for server to shut down automatically... ");

    while server.is_running() {
        sleep(Duration::new(0, 1000000));
    }
    println!("Server stopped");
}