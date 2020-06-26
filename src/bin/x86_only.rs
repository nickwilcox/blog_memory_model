#![allow(unused)]
use std::cell::UnsafeCell;
use std::sync::atomic::*;
use std::sync::Arc;
use std::thread;
use std::u32;

pub struct SynchronisedSum {
    shared: UnsafeCell<*const u32>,
    samples: usize,
}

impl SynchronisedSum {
    pub fn new(samples: usize) -> Self {
        assert!((samples as u32) < u32::MAX);
        Self {
            shared: UnsafeCell::new(std::ptr::null()),
            samples,
        }
    }

    #[inline(never)]
    pub fn generate(&self) {
        let data: Box<[u32]> = (0..self.samples as u32).collect();

        let shared_ptr = self.shared.get();
        unsafe {
            shared_ptr.write_volatile(data.as_ptr());
        }
        std::mem::forget(data);
    }

    #[inline(never)]
    pub fn calculate(&self, expected_sum: u32) {
        loop {
            // let shared_ptr = ;
            let data_ptr = unsafe { self.shared.get().read_volatile() };

            if !data_ptr.is_null() {
                let data = unsafe { std::slice::from_raw_parts(data_ptr, self.samples) };
                let mut sum = 0;
                for i in (0..data.len()).rev() {
                    sum += data[i];
                }
                assert_eq!(sum, expected_sum);
                break;
            }
        }
    }
}

// leaking memory makes the race condition occur more frequently
// impl Drop for SynchronisedSum {
//     fn drop(&mut self) {

//         unsafe {
//             let shared_ptr = self.shared.get();
//             let data_ptr = *shared_ptr as *mut u32;
//             if !data_ptr.is_null() {
//                 Box::from_raw(std::slice::from_raw_parts_mut(data_ptr, self.samples));
//             }
//         }
//     }
// }

unsafe impl Send for SynchronisedSum {}
unsafe impl Sync for SynchronisedSum {}

fn print_arch() {
    if cfg!(target_arch = "x86_64") {
        println!("running on x86_64");
    } else if cfg!(target_arch = "x86") {
        println!("running on x86");
    } else if cfg!(target_arch = "aarch64") {
        println!("running on aarch64");
    } else if cfg!(target_arch = "arm") {
        println!("running on arm");
    } else {
        println!("running on unknown!");
    }
}

pub fn main() {
    print_arch();
    for i in 0..10_000 {
        let sum_generate = Arc::new(SynchronisedSum::new(512));
        let sum_calculate = Arc::clone(&sum_generate);
        let calculate_thread = thread::spawn(move || {
            sum_calculate.calculate(130816);
        });
        thread::sleep(std::time::Duration::from_millis(1));
        let generate_thread = thread::spawn(move || {
            sum_generate.generate();
        });

        calculate_thread
            .join()
            .expect(&format!("iteration {} failed", i));
        generate_thread.join().unwrap();
    }
    println!("all iterations passed");
}
