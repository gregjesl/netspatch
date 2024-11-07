use netspatch::{client::Client, job::JobManager, server::Server};
use std::{sync::{Arc,Mutex}, thread::sleep, time::Duration};

fn main() {
    let host = "localhost".to_string();
    let port = 7878;

    let stack = Arc::new(
        Mutex::new(JobManager::new(&vec![2,2]).expect("Could not create stack")
    ));

    // Create the server
    let server = Server::start(&host, port, stack).expect("Could not start server");
    println!("Server started");

    // Wait
    sleep(Duration::new(1, 0));

    // Create a client
    let mut client = Client::connect(&host, port).expect("Could not connect to server");
    println!("Client connected");

    // Wait
    sleep(Duration::new(1, 0));

    // Request job
    client.get_job().expect("Failed to get job");
    
    // Wait
    sleep(Duration::new(1, 0));

    // Disconnect
    client.disconnect().expect("Error disconnecting client");
    println!("Client disconnected");

    // Wait
    sleep(Duration::new(1, 0));

    // Shutdown
    server.stop().expect("Could not stop server");
    println!("Server stopped");
}