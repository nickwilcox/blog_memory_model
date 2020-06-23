use std::sync::atomic::*;
use std::sync::Arc;
use std::thread;

struct SynchronisedSum {
    data: *const u32,
    samples: usize,
}

impl SynchronisedSum {
    fn new() -> Self {
        Self {
            data: std::ptr::null(),
            samples: 512,
        }
    }

    #[inline(never)]
    fn generate(&self) {
        let data = (0..self.samples as u32).collect::<Box<[u32]>>();

        let dst_ptr = &self.data as *const _ as *mut *const u32;
        unsafe {
            dst_ptr.write_volatile(data.as_ptr());
        }
        std::mem::forget(data);
    }

    #[inline(never)]
    fn consume(&self) {
        loop {
            let src_ptr = &self.data as *const *const u32;
            let data_ptr = unsafe { src_ptr.read_volatile() };

            if !data_ptr.is_null() {
                let mut sum = 0;
                for i in (0..self.samples).rev() {
                    sum += unsafe { data_ptr.offset(i as isize).read() };
                }
                assert_eq!(sum, 130816);
                break;
            }
        }
    }
}

impl Drop for SynchronisedSum {
    fn drop(&mut self) {
        // unsafe {
        //     Box::from_raw(std::slice::from_raw_parts_mut(
        //         self.data as *mut u32,
        //         self.samples,
        //     ));
        // }
    }
}

unsafe impl Send for SynchronisedSum {}
unsafe impl Sync for SynchronisedSum {}

struct SynchronisedSumFixed {
    data: AtomicPtr<u32>,
    samples: usize,
}

impl SynchronisedSumFixed {
    fn new() -> Self {
        Self {
            data: AtomicPtr::new(std::ptr::null_mut()),
            samples: 512,
        }
    }

    #[inline(never)]
    fn generate(&self) {
        let mut data = (0..self.samples as u32).collect::<Box<[u32]>>();

        self.data.store(data.as_mut_ptr(), Ordering::Relaxed);

        std::mem::forget(data);
    }

    #[inline(never)]
    fn consume(&self) {
        loop {
            let data_ptr = self.data.load(Ordering::Acquire);

            if !data_ptr.is_null() {
                let mut sum = 0;
                for i in (0..self.samples).rev() {
                    sum += unsafe { data_ptr.offset(i as isize).read() };
                }
                assert_eq!(sum, 130816);
                break;
            }
        }
    }
}

pub fn main() {
    for _ in 0..10000 {
        let sum_generate = Arc::new(SynchronisedSumFixed::new());
        let sum_consume = Arc::clone(&sum_generate);
        let consume_thread = thread::spawn(move || {
            sum_consume.consume();
        });
        thread::sleep(std::time::Duration::from_millis(1));
        let generate_thread = thread::spawn(move || {
            sum_generate.generate();
        });

        generate_thread.join().unwrap();
        consume_thread.join().unwrap();
    }
}
