use std::{thread, time::Duration, sync::{Arc, atomic::{Ordering, AtomicUsize}}};

use super::{ShrinkPool, SyncThread};
//I don't know how to test them. Printlns are nice but they are not unit tests.
#[test]
fn shrink_pool_test_sync() -> Result<(), String> {
    let pool = ShrinkPool::new(1);

    for i in 0..10 {
        pool.execute(move || {
            println!("id {:?} num {}", thread::current().id(), i);
        })
    }
    Ok(())
}

#[test]
fn sync_thread_test_sync() -> Result<(), String> {
    let pool = SyncThread::new();

    for i in 0..10 {
        pool.execute(move || {
            println!("id {:?} num {}", thread::current().id(), i);
        })
    }
    Ok(())
}

#[test]
fn shrink_pool_test_pooled() -> Result<(), String> {
    let pool = ShrinkPool::new(8);

    for i in 0..20 {
        pool.execute(move || {
            println!("id {:?} num {}", thread::current().id(), i);
        })
    }
    Ok(())
}

#[test]
fn shrink_pool_test_pooled_and_pause() -> Result<(), String> {
    let pool = ShrinkPool::new(4);

    for i in 0..20 {
        pool.execute(move || {
            println!("id {:?} num {}", thread::current().id(), i);
        })
    }
    thread::sleep(Duration::from_secs(2));
    println!("paused");

    for i in 0..20 {
        pool.execute(move || {
            println!("id {:?} num {}", thread::current().id(), i);
        })
    }
    thread::sleep(Duration::from_secs(2));
    Ok(())
}
#[test]
fn shrink_pool_test_panicked() -> Result<(), String> {
    let pool = ShrinkPool::new(8);

    for i in 0..50 {
        pool.execute(move || {
            if i % 5 == 0 {
                println!("");
                println!("panic is preparing...");
                panic!("panicked id {:?} num {}", thread::current().id(), i);
            } else {
                println!("");
                println!("success id {:?} num {}", thread::current().id(), i);
                println!("");
            }
        })
    }
    thread::sleep(Duration::from_secs(5));
    Ok(())
}

#[test]
fn sync_thread_test_panicked() -> Result<(), String> {
    let pool = SyncThread::new();

    for i in 0..50 {
        pool.execute(move || {
            if i % 5 == 0 {
                println!("");
                println!("panic is preparing...");
                panic!("panicked id {:?} num {}", thread::current().id(), i);
            } else {
                println!("");
                println!("success id {:?} num {}", thread::current().id(), i);
                println!("");
            }
        })
    }
    thread::sleep(Duration::from_secs(5));
    Ok(())
}

#[test]
fn shrink_pool_concurrency_test(){
    let counter = Arc::new(AtomicUsize::new(0));
    let pool = Arc::new(ShrinkPool::new(num_cpus::get()));
    let mut handles = vec![];
    for _ in 0..100{
        let counter = counter.clone();
        let pool = pool.clone();
        let handle = thread::spawn(move ||{
            pool.execute(move ||{
                counter.fetch_add(1, Ordering::Relaxed);
            });
        });
        handles.push(handle);
    }
    for handle in handles{
        let _unused = handle.join();
    }
    assert_eq!(counter.load(Ordering::Relaxed), 100);
}

#[test]
fn sync_thread_concurrency_test(){
    let counter = Arc::new(AtomicUsize::new(0));
    let thread = Arc::new(SyncThread::new());
    let mut handles = vec![];
    for _ in 0..100{
        let counter = counter.clone();
        let thread = thread.clone();
        let handle = thread::spawn(move ||{
            thread.execute(move ||{
                counter.fetch_add(1, Ordering::Relaxed);
            });
        });
        handles.push(handle);
    }
    for handle in handles{
        let _unused = handle.join();
    }
    assert_eq!(counter.load(Ordering::Relaxed), 100);
}
#[test]
fn typical_usecase() {
    use crate::ShrinkPool;
    use num_cpus;
    let pool = ShrinkPool::new(num_cpus::get());

    for i in 0..10 {
        pool.execute(move || println!("Task {i} is processing..."))
    }
}

#[test]
fn typical_usecase_sync_thread() {
    use crate::SyncThread;
    
    let thread = SyncThread::new();

    for i in 0..10 {
        thread.execute(move || print!("{i},"))
    }
}
