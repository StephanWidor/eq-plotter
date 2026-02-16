use atomic_enum::*;
use std::cell::{Cell, UnsafeCell};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Create a swap instance, and return a Producer and Consumer pair, that share the swap for data exchange.
/// \param init_value The initial value for the swap data. This will be cloned for each page.
///        Note that cloning a Vec::from_capacity() will not clone the capacity, so if you want to have a specific capacity for the swap data,
///        you should use make_swap_with_init_function instead.
pub fn make_swap_with_init_value<T: Send + Clone>(init_value: &T) -> (Producer<T>, Consumer<T>) {
    make_swap_with_init_function(&|| init_value.clone())
}

/// Create a swap instance, and return a Producer and Consumer pair, that share the swap for data exchange.
/// \param init_function A function that will be called to create the initial data for each page.
pub fn make_swap_with_init_function<T: Send + Clone>(
    init_function: &impl Fn() -> T,
) -> (Producer<T>, Consumer<T>) {
    let shared = Arc::new(Shared {
        states: [
            AtomicDataState::new(DataState::Writing),
            AtomicDataState::new(DataState::Reading),
            AtomicDataState::new(DataState::Free),
        ],
        data: std::array::from_fn(|_| UnsafeCell::new(init_function())),
    });

    let producer = Producer {
        shared: shared.clone(),
        writing_page: Cell::new(0),
        written_page: Cell::new(1),
    };

    let consumer = Consumer {
        shared: shared.clone(),
        reading_page: Cell::new(1),
    };

    (producer, consumer)
}

impl<T: Clone + Send> Producer<T> {
    /// Set a new producer value.
    /// After this, you still need to call push() to get it over to the consumer side.
    pub fn set(&self, value: T) {
        *self.acquire_writing_data() = value;
    }

    /// Set a new producer value, and push it to the consumer side.
    pub fn set_and_push(&self, value: T) {
        self.set(value);
        self.push();
    }

    /// Manipulate the current producer value.
    /// After this, you still need to call push() to get it over to the consumer side.
    pub fn manipulate(&self, callback: &impl Fn(&mut T)) {
        callback(self.acquire_writing_data());
    }

    /// Manipulate the current producer value, and push it to the consumer side
    pub fn manipulate_and_push(&self, callback: &impl Fn(&mut T)) {
        self.manipulate(callback);
        self.push();
    }

    /// Push current producer value to the consumer side.
    pub fn push(&self) {
        let current_writing_page = self.writing_page.get();
        if current_writing_page > 2 {
            return;
        }
        let last_written_page = self.written_page.get();
        let _ = self.shared.states[last_written_page]
            .compare_exchange_weak(
                DataState::Written,
                DataState::Free,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
        self.shared.states[current_writing_page].store(DataState::Written, Ordering::Release);
        self.written_page.set(current_writing_page);
        self.writing_page.set(3);
    }

    fn acquire_writing_data(&self) -> &mut T {
        let current_writing_page = self.writing_page.get();
        if current_writing_page <= 2 {
            // Seems we haven't pushed yet, so let's just return the current writing page
            return unsafe { &mut *self.shared.data[current_writing_page].get() };
        }

        // If we find a free page, we will use that as new writing page
        for i in 0..3 {
            if self.shared.states[i].load(Ordering::Acquire) == DataState::Free {
                self.shared.states[i].store(DataState::Writing, Ordering::Relaxed);
                self.writing_page.set(i);
                return unsafe { &mut *self.shared.data[i].get() };
            }
        }

        // We haven't found a free page (i.e. written page hasn't been consumed yet), so let's see if we can reuse the last written page
        let last_written_page = self.written_page.get();
        if self.shared.states[last_written_page]
            .compare_exchange_weak(
                DataState::Written,
                DataState::Writing,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            self.writing_page.set(last_written_page);
            return unsafe { &mut *self.shared.data[last_written_page].get() };
        }

        // We can still get here, if consumer grabbed the written page just before we wanted to reuse it.
        // In that case, the consumer will free the old reading page within its current pull, so we just loop until that happened.
        loop {
            for i in 0..3 {
                if self.shared.states[i].load(Ordering::Acquire) == DataState::Free {
                    self.shared.states[i].store(DataState::Writing, Ordering::Relaxed);
                    self.writing_page.set(i);
                    return unsafe { &mut *self.shared.data[i].get() };
                }
            }
        }
    }
}

impl<T: Clone + Send> Consumer<T> {
    /// Pull new data that has been sent/pushed from producer side.
    pub fn pull(&self) {
        let old_reading_page = self.reading_page.get();
        for i in 0..3 {
            if self.shared.states[i]
                .compare_exchange_weak(
                    DataState::Written,
                    DataState::Reading,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                self.shared.states[old_reading_page].store(DataState::Free, Ordering::Release);
                self.reading_page.set(i);
                break;
            }
        }
    }

    /// Read (copy of) the current consumer value.
    /// If you want to get the latest value pushed from producer, call pull() before this, or use pull_and_read().
    pub fn read(&self) -> T {
        unsafe { (*self.shared.data[self.reading_page.get()].get()).clone() }
    }

    /// Pull data, and read (copy of) the consumer value.
    pub fn pull_and_read(&self) -> T {
        self.pull();
        self.read()
    }

    /// Consume the current consumer value, i.e. do something with the received data.
    /// If you want to get the latest value pushed from producer, call pull() before this, or use pull_and_consume().
    pub fn consume(&self, callback: &mut impl FnMut(&T)) {
        let consumed = unsafe { &*self.shared.data[self.reading_page.get()].get() };
        callback(consumed);
    }

    /// Pull data, and consume the current consumer value, i.e. do something with the received data.
    pub fn pull_and_consume(&self, callback: &mut impl FnMut(&T)) {
        self.pull();
        self.consume(callback);
    }
}

#[atomic_enum]
#[derive(PartialEq)]
enum DataState {
    Free = 0,
    Writing = 1,
    Written = 2,
    Reading = 3,
}

struct Shared<T> {
    states: [AtomicDataState; 3],
    data: [UnsafeCell<T>; 3],
}
unsafe impl<T: Send> Sync for Shared<T> {}

pub struct Producer<T: Send> {
    shared: Arc<Shared<T>>,
    writing_page: Cell<usize>,
    written_page: Cell<usize>,
}
unsafe impl<T: Send> Sync for Producer<T> {}

pub struct Consumer<T: Send> {
    shared: Arc<Shared<T>>,
    reading_page: Cell<usize>,
}
unsafe impl<T: Send> Sync for Consumer<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::assert_ge;
    use std::sync::atomic::AtomicBool;

    fn test_feeding_and_consuming(
        feeding_wait_time: std::time::Duration,
        consuming_wait_time: std::time::Duration,
        run_time: std::time::Duration,
    ) {
        const ARRAY_SIZE: usize = 50;
        let (producer, consumer) = make_swap_with_init_value(&[0; ARRAY_SIZE]);
        let keep_running = Arc::new(AtomicBool::new(true));

        let keep_running_for_feeding = keep_running.clone();
        let feeding_thread_handle = std::thread::spawn(move || {
            let mut i = 0;
            while keep_running_for_feeding.load(Ordering::Relaxed) {
                i += 1;
                producer.manipulate_and_push(&mut |array| array.fill(i));
                if !feeding_wait_time.is_zero() {
                    std::thread::sleep(feeding_wait_time);
                }
            }
        });
        let keep_running_for_consuming = keep_running.clone();
        let consuming_thread_handle = std::thread::spawn(move || {
            let mut last_value = 0;
            while keep_running_for_consuming.load(Ordering::Relaxed) {
                consumer.pull_and_consume(&mut |consumed| {
                    let start_value = consumed.first().unwrap().clone();
                    assert_ge!(start_value, last_value);
                    if start_value < last_value {
                        return;
                    }
                    for i in consumed {
                        assert_eq!(*i, start_value);
                        if *i != start_value {
                            return;
                        }
                    }
                    last_value = start_value;
                    if !consuming_wait_time.is_zero() {
                        std::thread::sleep(consuming_wait_time);
                    }
                });
            }
        });

        std::thread::sleep(run_time);
        keep_running.store(false, Ordering::Relaxed);
        feeding_thread_handle
            .join()
            .expect("Couldn't join feeding thread");
        consuming_thread_handle
            .join()
            .expect("Couldn't join consuming thread");
    }

    #[test]
    fn fast_feeding_and_consuming() {
        test_feeding_and_consuming(
            std::time::Duration::ZERO,
            std::time::Duration::ZERO,
            std::time::Duration::from_secs(3),
        );
    }

    #[test]
    fn slower_feeding() {
        test_feeding_and_consuming(
            std::time::Duration::from_millis(10),
            std::time::Duration::ZERO,
            std::time::Duration::from_secs(3),
        );
    }

    #[test]
    fn slower_consuming() {
        test_feeding_and_consuming(
            std::time::Duration::ZERO,
            std::time::Duration::from_millis(10),
            std::time::Duration::from_secs(3),
        );
    }
}
