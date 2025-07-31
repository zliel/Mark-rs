use std::{
    fmt,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use log::warn;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<Self, Error> {
        if size == 0 {
            return Err(Error::PoolCreation {
                message: "Thread pool size must be greater than 0".to_string(),
            });
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::build(id, Arc::clone(&receiver)).map_err(|e| {
                Error::PoolCreation {
                    message: format!("Failed to create worker thread {}: {}", id, e),
                }
            })?);
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn join_all(self) {
        drop(self.sender); // Close the channel to signal workers to exit

        for worker in self.workers {
            if let Err(e) = worker.thread.join() {
                warn!("Worker thread {} failed to join: {:?}", worker.id, e);
            }
        }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        self.sender.send(job).map_err(|e| {
            warn!("Failed to send job to thread pool: {e}");
            Error::JobExecution {
                message: format!("Failed to send job to thread pool: {e}"),
            }
        })
    }
}

struct Worker {
    pub id: usize,
    pub thread: thread::JoinHandle<()>,
}

impl Worker {
    fn build(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Self, Error> {
        let builder = thread::Builder::new();

        let thread = builder
            .spawn(move || {
                loop {
                    let job_result = {
                        let receiver = receiver.lock().expect("Failed to lock receiver mutex");
                        receiver.recv()
                    };

                    match job_result {
                        Ok(job) => {
                            job();
                        }
                        Err(_) => {
                            break; // Exit the loop if the channel is closed
                        }
                    }
                }
            })
            .map_err(|e| Error::WorkerCreation {
                message: format!("Failed to spawn thread {id}: {e}"),
            })?;

        Ok(Worker { id, thread })
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub enum Error {
    PoolCreation { message: String },
    JobExecution { message: String },
    WorkerCreation { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PoolCreation { message } => write!(f, "Thread pool creation error: {message}"),
            Error::JobExecution { message } => write!(f, "Job execution error: {message}"),
            Error::WorkerCreation { message } => write!(f, "Worker creation error: {message}"),
        }
    }
}

impl std::error::Error for Error {}
