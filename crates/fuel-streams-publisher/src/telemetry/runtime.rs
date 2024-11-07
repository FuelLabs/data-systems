use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Arc, Mutex},
};

use futures::Future;
use tokio::time::{self, Duration};

// Task type: Each task is represented by a Boxed, pinned Future
type Task = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone)]
pub struct Runtime {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    max_capacity: usize,
    interval: Duration,
}

impl Runtime {
    pub fn new(capacity: usize, interval: Duration) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            max_capacity: capacity,
            interval,
        }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut queue = self.task_queue.lock().unwrap();

        // If the queue is at capacity, discard the oldest task
        if queue.len() >= self.max_capacity {
            queue.pop_front();
        }

        queue.push_back(Box::pin(task));
    }

    pub fn start(&self) {
        let interval = self.interval;
        let task_queue = Arc::clone(&self.task_queue);

        tokio::spawn(async move {
            let mut ticker = time::interval(interval);

            loop {
                // Wait for the interval
                ticker.tick().await;

                // Lock the queue, drain tasks, and run them sequentially
                let tasks: Vec<_> = {
                    let mut queue = task_queue.lock().unwrap();
                    queue.drain(..).collect()
                };

                // Run each task sequentially
                for task in tasks {
                    task.await;
                }
            }
        });
    }
}
