mod lib;
use lib::WaitFreeVector;
use std::sync::Arc;
use std::thread;
use rand::Rng;
use std::time::{Instant, Duration};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_vals_seq(){
        let vec = WaitFreeVector::new(3);
        vec.push_back(0, 10);
        vec.push_back(0, 11);
        vec.push_back(0, 12);
    }
    #[test]
    fn len_seq(){
        let vec = WaitFreeVector::new(3);
        vec.push_back(0, 10);
        vec.push_back(0, 11);
        assert_eq!(vec.length(), 2);
    }
    // #[test]
    // #[should_panic(expected = "Mem Overflow")]
    // fn memory_overflow(){
    //     let vec = WaitFreeVector::new(2);
    //     vec.push_back(0, 10);
    //     vec.push_back(0, 11);
    //     vec.push_back(0, 12);
    // }

    #[test]
    fn seq_at(){
        let vec = WaitFreeVector::new(2);
        vec.push_back(0, 10);
        vec.at(0, 0);
    }

    #[test]
    fn threaded_insert_len(){
        let capacity = 100;
        let num_threads = 8;
        let times = 12;
        assert!(num_threads*times < capacity);
        
        let vec = Arc::new(WaitFreeVector::new(100));
        let mut handles = Vec::new();
        
        for i in 0..num_threads {

            let vec_thread = vec.clone();
            handles.push(
                thread::spawn(
                    move || {
                        for _ in 0..times {
                            vec_thread.push_back(i, i*i);
                        }
                    }
                )
            );
        }

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(vec.length(), num_threads * times);
        }

    #[test]
    fn threaded_resize(){
        let capacity = 1;
        let num_threads = 10;
        let times = 10;
        assert!(num_threads*times > capacity);
        
        let vec = Arc::new(WaitFreeVector::new(100));
        let mut handles = Vec::new();
        
        for i in 0..num_threads {

            let vec_thread = vec.clone();
            handles.push(
                thread::spawn(
                    move || {
                        for _ in 0..times {
                            vec_thread.push_back(i, i*i);
                        }
                    }
                )
            );
        }

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(vec.length(), num_threads * times);
        }
}



fn main(){

    let vec = WaitFreeVector::new(3);
    vec.push_back(0, 1);
    vec.push_back(0, 2);
    vec.push_back(0, 2);
    // vec.resize();
    vec.push_back(0, 2);
    println!("ligma");


}