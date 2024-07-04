use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    id: u32,
}

impl Worker {
    pub fn new<F: FnOnce() + Send + 'static>(
        id: u32,
        queue: Arc<Mutex<Option<VecDeque<F>>>>,
    ) -> Self {
        let thread = thread::spawn(move || loop {
            let mut guard = queue.lock().unwrap();
            let queue = match guard.as_mut() {
                Some(queue) => queue,
                None => break,
            };

            match queue.pop_front() {
                None => (),
                Some(job) => job(),
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool<F: FnOnce() + Send + 'static> {
    jobs: Arc<Mutex<Option<VecDeque<F>>>>,
    workers: Vec<Worker>,
}

impl<F: FnOnce() + Send + 'static> std::fmt::Display for ThreadPool<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::with_capacity(self.workers.len());
        for worker in &self.workers {
            v.push(worker.id);
        }

        write!(f, "currently_running_threads: {v:?}")
    }
}

impl<F: FnOnce() + Send + 'static> ThreadPool<F> {
    pub fn new(n: usize) -> Result<Self, ()> {
        if n == 0 {
            return Err(());
        }

        let jobs = Arc::new(Mutex::new(Some(VecDeque::with_capacity(n))));
        let mut workers = Vec::with_capacity(n);
        for id in 0..n {
            let worker = Worker::new(id as u32, Arc::clone(&jobs));
            workers.push(worker);
        }

        Ok(ThreadPool { jobs, workers })
    }

    pub fn spawn(&mut self, job: F) {
        let guard = self.jobs.lock();
        match guard {
            Err(_) => return,
            Ok(mut queue) => queue.as_mut().unwrap().push_back(job),
        }
    }
}

impl<F: FnOnce() + Send + 'static> Drop for ThreadPool<F> {
    fn drop(&mut self) {
        drop(self.jobs.lock().unwrap().take());

        for worker in &mut self.workers {
            match worker.thread.take() {
                None => (),
                Some(thread) => thread.join().unwrap(),
            }
        }

        return ();
    }
}
