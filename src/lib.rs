use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0); // checks for the numebers of threads should be greater than zero
        let (sender, receiver) = mpsc::channel();

        // we need to have shere owneship and mutability of the receiver
        // so we use the smart pointer ARC for multiple ownwership
        // and Mutex thread safe behaviour
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            //create the threads
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            threads: workers,
            sender,
        }
    }

    // We still use the () after FnOnce because this FnOnce
    // represents a closure that takes no parameters and returns the unit type
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Send terminate message to all workers...");
        for _ in &self.threads {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Terminate to all workers...");
        // since that is async we need to join all the
        // work in order to let each thread to get the terminate msg
        // and complete their task
        for worker in &mut self.threads {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            };
        }
    }
}
// A Worker Struct Responsible for Sending Code from the ThreadPool to a Thread
// in our case, we want to create the threads and have them wait for code that we’ll send later.

/*
The Worker picks up code that needs to be run and runs the code in the Worker’s thread.
Think of people working in the kitchen at a restaurant: the workers wait until orders come in from customers,
and then they’re responsible for taking those orders and fulfilling them.
Instead of storing a vector of JoinHandle<()> instances in the thread pool,
we’ll store instances of the Worker struct. Each Worker will store a single JoinHandle<()> instance.
*/
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        /* In the worker, our closure being passed to thread::spawn still only references the receiving end of the channel.
        Instead, we need the closure to loop forever, asking the receiving end of the channel for a job and
        running the job when it gets one. */
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock() // to adquire the mutex
                .unwrap() // if anything fails
                .recv() // to receive the job from channel
                .unwrap(); // again checks if fails

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job, executing ", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} to terminate executing ", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
