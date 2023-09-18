use std::{
    collections::VecDeque,
    future::Future,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};
use thiserror::Error;
/// A thread pool which agressively terminates its threads as soon as they are idle.
/// If there are queued tasks, OS threads are spawned until num_threads >= pool_size.
/// When all tasks have been done, no threads are running on this pool.
/// The tasks start in a FIFO(First-In-First-Out) manner. No workstealing occurs.
/// However, the order in which tasks are completed depends on the OS.
pub struct ShrinkPool {
    pool_size: usize,
    mutex: Arc<Mutex<ShrinkPoolInner>>,
}

struct ShrinkPoolInner {
    num_running_threads: usize,
    tasks: VecDeque<Box<dyn FnOnce() + Send + 'static>>,
}

impl ShrinkPool {
    /// Panics when pool_size is 0.
    pub fn new(pool_size: usize) -> ShrinkPool {
        if pool_size == 0 {
            panic!("pool_size can't be zero.")
        }
        ShrinkPool {
            pool_size,
            mutex: Arc::new(Mutex::new(ShrinkPoolInner {
                num_running_threads: 0,
                tasks: VecDeque::new(),
            })),
        }
    }

    /// Execute a task. When the task is panicked, the task is discarded and the thread is silently respawned if the panic can be unwinded, and the remaining tasks will be processed.
    /// In Rust, there are panics which can't be unwinded. When the panic occur, the current process will be aborted, so we can do nothing.
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let spawn = {
            //When this mutex is poisoned, I believe this pool shouldn't keep running. When memory is insufficient, it can be poisoned.
            let mut inner = self.mutex.lock().expect("mutex is poisoned");

            //This can panic when the memory is insufficient.
            //At least this panic occurs in the current thread and the app will be notified.
            //When a panic occured in a thread of this pool, the app might not be notified and it may cause complicated problems.
            inner.tasks.push_back(Box::new(f));
            if inner.num_running_threads < self.pool_size {
                inner.num_running_threads += 1;
                true
            } else {
                false
            }
        };
        if spawn {
            let cloned = self.mutex.clone();
            thread_spawn(cloned);
        }
    }
}

fn thread_spawn(cloned: Arc<Mutex<ShrinkPoolInner>>) {
    thread::spawn(move || loop {
        let f = {
            //When this mutex is poisoned, I believe this pool shouldn't keep running.
            let mut inner = cloned.lock().expect("mutex is poisoned");
            match inner.tasks.pop_front() {
                Some(f) => f,
                None => {
                    inner.num_running_threads -= 1;
                    break;
                }
            }
        };
        //When the mutex is poisoned, the code above will panic,
        //so PanicCatcher won't be constructed.
        
        let mut catcher = PanicCatcher {
            mutex: cloned.clone(),
            is_working: true,
        };
        //When f() panics, the mutex won't be poisoned because the MutexGuard already dropped.
        f();
        catcher.is_working = false;
    });
}

struct PanicCatcher {
    mutex: Arc<Mutex<ShrinkPoolInner>>,
    is_working: bool,
}

impl Drop for PanicCatcher {
    fn drop(&mut self) {
        if self.is_working {
            //Respawn a thread. num_running_thread will not be inconsistent.
            //When only one thread is running, if it's panicked and not respawned, remaining tasks won't be run.
            //Therefore, respawn strategy is necessary, I believe.

            //When the mutex is poisoned, the spawned thread panics.
            //Make sure PanicCatcher isn't constructed in the thread to avoid infinite loop.
            thread_spawn(self.mutex.clone());
        }
    }
}

/// ShrinkPool whose size is 1.
/// This can synchronize tasks, which means tasks run in the order they are given, one by one.
/// The thread is terminated when it's idle, and respawned when a task is given.
pub struct SyncThread {
    pool: ShrinkPool,
}

impl SyncThread {
    /// Create a SyncThread. No thread runs at this point.
    pub fn new() -> SyncThread {
        SyncThread {
            pool: ShrinkPool::new(1),
        }
    }

    /// Execute a task in a FIFO(First-In-First-Out) manner. An OS thread is spawned if needed.
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.pool.execute(f)
    }
}
