use rand::prelude::*;
use std::sync::Arc;
use std::thread::{available_parallelism, scope, ScopedJoinHandle};

use std::time::Instant;

const STR_SIZE_MAX: usize = 1024;
const ITERATIONS: usize = 1 << 20;
const LOCAL_SCRATCH_SIZE: usize = 100 << 20;

fn print_header() {
    println!("arc_or_clone num_strings string_len threads operations seconds ops_per_sec");
}
fn print_data(
    tag: &str,
    res: &ExperimentResult,
    num_strings: usize,
    strlen: usize,
    threads: usize,
) {
    println!(
        "{} {} {} {} {} {:.3} {:.1}",
        tag,
        num_strings,
        strlen,
        threads,
        res.num_operations,
        res.elapsed_sec,
        res.num_operations as f64 / res.elapsed_sec
    )
}

#[derive(Default, Debug)]
struct ExperimentResult {
    num_operations: usize,
    elapsed_sec: f64,
}

impl ExperimentResult {
    fn new() -> Self {
        Self {
            num_operations: 0,
            elapsed_sec: 0.0,
        }
    }

    fn merge(&mut self, other: Self) {
        self.num_operations += other.num_operations;
        self.elapsed_sec += other.elapsed_sec;
    }
}

pub trait StringSrc {
    // TODO: one func w/ generic return type?
    fn get(&self) -> String;
    fn get_arc(&self) -> Arc<String>;
    fn want_arc(&self) -> bool; // yuck
}

#[derive(Debug)]
struct CloneStrSrc {
    string_pool: Vec<String>,
}

impl CloneStrSrc {
    fn new(strlen: usize, num_strs: usize) -> CloneStrSrc {
        let mut string_pool = Vec::with_capacity(num_strs);
        for _ in 0..num_strs {
            string_pool.push(random_string(strlen));
        }
        Self { string_pool }
    }
}

impl StringSrc for CloneStrSrc {
    fn get(&self) -> String {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.string_pool.len());
        self.string_pool[idx].clone()
    }

    fn get_arc(&self) -> Arc<String> {
        todo!()
    }

    fn want_arc(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct ArcStrSrc {
    string_pool: Vec<Arc<String>>,
}

impl ArcStrSrc {
    fn new(strlen: usize, num_strs: usize) -> ArcStrSrc {
        let mut string_pool = Vec::with_capacity(num_strs);
        for _ in 0..num_strs {
            string_pool.push(Arc::new(random_string(strlen)));
        }
        Self { string_pool }
    }
}

impl StringSrc for ArcStrSrc {
    fn get(&self) -> String {
        !todo!()
    }

    fn get_arc(&self) -> Arc<String> {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.string_pool.len());
        self.string_pool[idx].clone()
    }

    fn want_arc(&self) -> bool {
        true
    }
}

// Function to determine the number of cores this machine has.
fn get_hw_threads() -> usize {
    available_parallelism().unwrap().get()
}

fn main() {
    let nthread = get_hw_threads();
    let set_sizes: [usize; 3] = [1, 8, 128];
    print_header();
    for &n in set_sizes.iter() {
        test_num_strs(n, nthread);
    }
}

fn test_num_strs(n: usize, nthread: usize) {
    let total_threads = nthread * 4;
    let str_sizes: [usize; 5] = [16, 32, 64, 512, STR_SIZE_MAX];
    for &strlen in str_sizes.iter() {
        {
            let string_pool = ArcStrSrc::new(strlen, n);
            let res = experiment(&string_pool, nthread);
            print_data("A", &res, n, strlen, total_threads);
        }
        {
            let string_pool = CloneStrSrc::new(strlen, n);
            let res = experiment(&string_pool, nthread);
            print_data("C", &res, n, strlen, total_threads);
        }
    }
}

fn string_calculation(s: &str) -> Option<&str> {
    // get sha256 hash of s
    //digest(s)
    if s.chars().all(char::is_alphanumeric) {
        Some(s)
    } else {
        None
    }
}

fn thread_loop<T>(strings: &T, _child_num: usize) -> ExperimentResult
where
    T: StringSrc,
{
    let local_vec_size = LOCAL_SCRATCH_SIZE / STR_SIZE_MAX;
    // Thread should do stuff and access memory
    // - Request the "immutable" / shared string data.
    // - Do some computation depending its value.
    // - Touch other memory (local_mem) to simulate real app load.
    let mut local_mem: Vec<String> = Vec::with_capacity(local_vec_size);
    (0..local_vec_size).for_each(|_| {
        local_mem.push("".to_string());
    });

    let mut i = 0;
    let now = Instant::now();
    for _ in 0..ITERATIONS {
        // Get some stings for reading..
        if strings.want_arc() {
            let s = strings.get_arc();
            let output = string_calculation(&s);
            let idx = i % local_vec_size;
            if let Some(ostr) = output {
                local_mem[idx] = ostr.to_string();
                i += 1;
            }
        } else {
            let s = strings.get();
            let output = string_calculation(&s);
            let idx = i % local_vec_size;
            if let Some(ostr) = output {
                local_mem[idx] = ostr.to_string();
                i += 1;
            }
        }
    }
    let elapsed_sec = now.elapsed().as_secs_f64();
    ExperimentResult {
        num_operations: ITERATIONS,
        elapsed_sec,
    }
}

fn random_string(n: usize) -> String {
    let mut s = String::with_capacity(n);
    for _ in 0..n {
        s.push(random::<char>());
    }
    s
}

fn experiment<T>(strings: &T, num_threads: usize) -> ExperimentResult
where
    T: StringSrc + std::marker::Sync + std::marker::Send,
{
    // Spawn threads which run thread_loop.
    // Use ScopedJoinHandle to wait for all threads to finish.

    scope(|s| {
        let mut results: Vec<ScopedJoinHandle<ExperimentResult>> = Vec::with_capacity(num_threads);
        for i in 0..num_threads {
            let sjh = s.spawn(move || thread_loop(strings, i));
            results.push(sjh);
        }
        let mut result = ExperimentResult::new();
        for tr in results {
            result.merge(tr.join().unwrap());
        }
        result
    })
}
