use core::fmt;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            if worker.thread.join().is_err() {
                eprintln!("Could not join thread");
            } else {
                println!("Worker {} shut down", worker.id)
            }
        }
    }
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
    /// The size is the number of threads in the pool.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::ZeroThreads);
        }
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);
        for worker_id in 0..size {
            workers.push(Worker::new(worker_id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool {
            workers,
            sender: Some(tx),
        })
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let job = Box::new(f);
        if self
            .sender
            .as_ref()
            .expect("sender should be none before dropped")
            .send(job)
            .is_err()
        {
            println!("Could not send job to thread pool")
        }
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
            println!("Worker {id} started");
            loop {
                // unlock receiver
                let Ok(receiver) = receiver.lock() else {
                    eprintln!("poisoned mutex");
                    break;
                };

                let message = receiver.recv();

                match message {
                    Ok(next_job) => next_job(),
                    Err(_) => {
                        println!("Worker {id} disconnected. Shutting down...");
                        break;
                    }
                }
            }
        });
        Worker { id, thread }
    }
}
