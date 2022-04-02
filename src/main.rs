use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

const UNLOCKED: bool = false;
const LOCKED: bool = true;

struct Mutex<T> {
    is_locked: AtomicBool,
    v: UnsafeCell<T>
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self { is_locked: AtomicBool::new(UNLOCKED), v: UnsafeCell::new(t) }
    }

    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.is_locked.compare_exchange_weak(UNLOCKED, LOCKED, 
            Ordering::Acquire,Ordering::Relaxed).is_err() {
            while self.is_locked.load(Ordering::Relaxed) == LOCKED
            {
                std::thread::yield_now();
            }
        }

        // Safety: we hold the lock, so we create a mutable reference.
        let return_value = f(unsafe { &mut *self.v.get() });
        self.is_locked.store(UNLOCKED, Ordering::Release);
        return_value
    }
}

fn main() {
    let l: &'static _ = Box::leak(Box::new(Mutex::new(0u32)));
    let handles: Vec<_> = (0..100)
        .map(|_| {
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    l.with_lock(|v| { *v += 1; });
                }
            })
        }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(l.with_lock(|v| *v), 100 * 1000)
}
