//! Structure that is responsible for executing a closure in a thread
//!
//! Is used inside the Threadpool structure to contain a thread waiting for a task
//! that is sent through a channel by the Threadpool structure.

use super::threadpool::Job;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// The Worker structure contains an id to be identify and a thread.
pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates and initializes a new Worker aka Thread
    /// listening for requests from it's ThreadPool.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = match receiver.lock() {
                Ok(ref receiver) => receiver.recv(),
                Err(_) => {
                    // Channel has been closed by self's ThreadPool
                    // or another thread paniced while holding the lock.
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            };

            if let Ok(job) = message {
                println!("Worker {id} got a job; executing.");
                job();
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
