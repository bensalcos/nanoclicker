//! Deliberate bounded CPU load for isolated benchmarks, never production input.
use std::thread;

// RAII also stops load threads if input, capture or timeout validation fails.
pub struct Load {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
}
impl Load {
    pub fn start(workers: usize) -> Self {
        let mut load = Self {
            stop: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            threads: Vec::with_capacity(workers),
        };
        for _ in 0..workers {
            let stop = load.stop.clone();
            load.threads.push(thread::spawn(move || {
                let mut value = 1u64;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    value = std::hint::black_box(
                        value.wrapping_mul(6364136223846793005).wrapping_add(1),
                    );
                }
            }));
        }
        load
    }
}
impl Drop for Load {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        for worker in self.threads.drain(..) {
            let _ = worker.join();
        }
    }
}
