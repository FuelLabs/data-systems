use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Arc, Mutex},
};

use futures::Future;

// Task type: Each task is represented by a Boxed, pinned Future
type Task = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone)]
pub struct Runtime {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    max_capacity: usize,
}

impl Runtime {
    pub fn new(capacity: usize) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            max_capacity: capacity,
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
}
