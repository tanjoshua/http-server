use core::fmt;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

#[derive(Debug, Clone)]
pub enum PoolCreationError {
    ZeroThreads,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error creating pool")
    }
}
impl Error for PoolCreationError {}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        /// The size is the number of threads in the pool.
        return Err(PoolCreationError::ZeroThreads);
        if size == 0 {}
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);
        for worker_id in 0..size {
            workers.push(Worker::new(worker_id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool {
            workers,
            sender: tx,
        })
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        // improvement: use thread builder so that server won't panic
        let thread = thread::spawn(move || {
            loop {
                // unlock receiver
                let next_job = receiver.lock().unwrap().recv().unwrap();
                next_job();
            }
        });
        Worker { id, thread }
    }
}
