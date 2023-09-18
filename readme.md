# ShrinkPool

A thread pool which agressively terminates its threads as soon as they are idle.

If there are queued tasks, OS threads are spawned until the pool is full.

When all tasks have been done, no threads are running on the pool.

The tasks start in a FIFO(First-In-First-Out) manner. No workstealing occurs.

However, the order in which tasks are completed depends on the OS.

```Rust
use shrink_pool::ShrinkPool;
use num_cpus;

let pool = ShrinkPool::new(num_cpus::get());

for i in 0..10 {
    pool.execute(move || println!("task {i} is processing..."))
}
```
```
Result:
Task 0 is processing...
Task 2 is processing...
Task 5 is processing...
Task 6 is processing...
Task 7 is processing...
Task 8 is processing...
Task 9 is processing...
Task 3 is processing...
Task 4 is processing...
Task 1 is processing...
```
If you want to synchronize tasks, you can use SyncThread.

It's basically a thread pool which has only one thread, and the thread is terminated when it's not running.
```Rust
use shrink_pool::SyncThread;
   
let thread = SyncThread::new();

for i in 0..10 {
    thread.execute(move || print!("{i},"))
}
```
```
Result: 
0,1,2,3,4,5,6,7,8,9,
```
### Motivation

I don't like libralies which silently spawn global threads and make them wait.
I want to clean them up when they are not running.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](apache_license.txt) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](mit_license.txt) or http://opensource.org/licenses/MIT)

<!--
[![crates.io link](https://img.shields.io/crates/v/docchi.svg)](https://crates.io/crates/docchi)

-->
