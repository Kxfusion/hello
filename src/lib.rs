use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<Worker>,
}

struct Worker {
    id: usize,
    handler: Option<JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    /// where size is the number of threads in the pool.
    ///
    /// # Panics
    /// Will panic if size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for thread_id in 0..size {
            workers.push(Worker::new(thread_id, Arc::clone(&receiver)));
        }

        ThreadPool {
            sender: Some(sender),
            workers,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down {}", worker.id);

            if let Some(worker) = worker.handler.take() {
                worker.join().unwrap();
            }
        }
    }
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handler = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected");
                    break;
                }
            }
        });

        Worker {
            id,
            handler: Some(handler),
        }
    }
}
