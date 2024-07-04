//!  A Threadpool  structure models a group of spawned threads that are waiting and ready to handle a task.
//!
//! Threadpool structure contains a fixed number of workers ready to execute a task in  a closure.
//! Threadpool is generally used by a Sever for processing multiple requests of a Client in different threads.

use super::worker::Worker;
use crate::{server_err, ServerError};
use std::sync::{mpsc, Arc, Mutex};

/// The Threadpool structure contains a vector of workers waiting to handle a task and a sender to send them a job through a channel.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// The type Job is a box pointer that owns a heap allocation of a closure
/// that is going to be executed by a Worker
pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Instanciates a new ThreadPool with `size` amount of Workers.
    ///
    /// # Err
    ///
    /// This method will return ServerError if size is 0.
    ///
    pub fn build(size: usize) -> Result<ThreadPool, ServerError> {
        if size == 0 {
            return Err(server_err!("Can't build Threadpool with size 0"));
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Assigns the given clossure to the first available worker.
    /// If no threads are available at the moment then the first
    /// to finish to receive it will execute it.
    pub fn execute<F>(&self, f: F) -> Result<(), ServerError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.as_ref() {
            None => Err(server_err!("Threadpool's job channel is closed")),
            Some(sender) => match sender.send(job) {
                Err(_) => Err(server_err!("Sending job through channel")),
                Ok(_) => Ok(()),
            },
        }
    }
}

/// When the pool is dropped, the threads in the Workers make join to make sure they finished their work
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => (),
                    Err(_) => println!("Worker {} panicked", worker.id),
                };
            }
        }
    }
}
