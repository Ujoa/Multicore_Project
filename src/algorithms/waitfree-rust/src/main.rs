mod lib;
use lib::WaitFreeVector;


use std::sync::Arc;
use std::thread;
use rand::Rng;
use std::time::Instant;


fn test_pushback(num_threads: usize){

    println!("TEST PUSHBACK {} threads", num_threads);
    let size = (num_threads as usize) * 30;
    let v = Arc::new(WaitFreeVector::new(size, num_threads));
    let mut threads = Vec::new();


    for i in 0..num_threads{
        let thread_v = v.clone();
        threads.push(
            thread::spawn(move || {

                for j in 0..30{
                    thread_v.push_back(i as usize, ((i as usize)+1)*100+(j as usize));
                }
                
            })
        );
    }

    for t in threads{
        t.join().unwrap();
    }

    for i in 0..v.length(){
        println!("{}", v.at(0,i).unwrap());
    }
}

fn test_popback(num_threads: usize){
    let len = 30;
    let size = (num_threads as usize) * len;
    // let size = 1000;
    println!("TEST POPBACK {} threads", num_threads);

    let v = Arc::new(WaitFreeVector::new(size, num_threads));
    // let good: Arc<WaitFreeVector> = Arc::new(WaitFreeVector::new(num_threads as usize));
    let mut threads = Vec::new();

    for i in 0..size{
        v.push_back(0, i as usize);
    }

    for i in 0..num_threads{
        let v_thread  = v.clone();

        
        threads.push(
                thread::spawn( move || {
                    let mut res = 0;
                    for _ in 0..len{
                        // let val = 10;
                        if let Some(val) = v_thread.pop_back(i) {
                            res += val;
                        }
                    }
                    return res;
                }
            )
        );
    }

    for t in threads {
        let res = t.join().unwrap();
        println!("{}", res );
    }

    for i in 0..v.length(){
        println!("{}", v.at(0,i).unwrap());
    }

    if v.length() > 0 {
        match v.pop_back(0) {
            Some(val) => {
                println!("res-pop_back{}", val);
            }
            None => {
                println!("none from pop_back");
            }
        }
    }

    println!("\n");
}


fn test_cwrite(num_threads: usize){

    let len = 44;
    let size = len * (num_threads as usize);

    let v = Arc::new(WaitFreeVector::new(size, num_threads));
    let mut threads = Vec::new();
    for _ in 0..len{
        v.push_back(0,0);
    }

    let mut cnt = Vec::new();
    for _ in 0..num_threads{
        let new_v = Arc::new(WaitFreeVector::new(len, num_threads));
        for _ in 0..len {
            new_v.push_back(0,0);
        }
        cnt.push(new_v);
    }

    for i in 1..num_threads {
        let thread_v = v.clone();
        let thread_cnt = cnt[i as usize].clone();
        threads.push(
            thread::spawn( move || {

                for j in 0..1000 {
                    let pos = j % thread_v.length();
                    let prev = thread_v.at(i as usize, pos).unwrap();

                    todo!("Implement CWrite and AddAt");
                    // if thread_v.cwrite(pos,prev, prev+1) {
                    //     thread_cnt.addat(pos, 1);
                    // }
                }
                }
            )
        );
    }

    for t in threads {
        t.join().unwrap();
    }

    let mut tot: Vec<usize> = vec![0;len];

    for i in 0..num_threads {
        for j in 0..len {
            let val = cnt[i as usize].at(i as usize, j).unwrap();
            tot[j] += val;
            println!("{} ", val);
        }
        println!();
    }
    println!("-------------");

    for i in 0..len{
        println!("{}", v.at(0, i).unwrap());
    }

    println!();

}


fn test_all(max_num_threads: usize){

    let max_ops = 6400;
    let insert = 0;
    let erase = 1;
    let limit = 25;

    for num_threads in 1..max_num_threads {

        print!("{}",num_threads);

        for t in vec![insert, erase]{
            let v = Arc::new(WaitFreeVector::new(num_threads+1, num_threads));

            let each_thread = max_ops/num_threads;
            let extra = max_ops % num_threads;
            let mut ops_per_thread = vec![num_threads+1];

            for i in 1..num_threads+1 {
                ops_per_thread.insert(i, each_thread);

                if i <= extra{
                    ops_per_thread[i] += 1;
                }
            }

            let start_time = Instant::now();
            for i in 0..10 {
                v.push_back(0,i);
            }

            let mut threads = Vec::new();

            for i in 1..num_threads{
                let thread_ops_per_thread = ops_per_thread.clone();
                let thread_v = v.clone();

                if t == insert {
                    threads.push(
                        thread::spawn(
                        move || {
                            let mut rng = rand::thread_rng();
                            let mut r = || -> usize {
                                rng.gen()
                            };

                            //TODO: Check why None value
                            // i.e why this wasn't working
                            //let tot_ops = thread_ops_per_thread.at(i).unwrap();
                            let tot_ops = thread_ops_per_thread[i];

                            for _ in 0..tot_ops {
                                let cur_op = r() % 3;
                                let do_pushack = (r()%100+100)%100 < limit;

                                let x = r();
                                let size = thread_v.length();

                                if do_pushack {
                                    thread_v.push_back(i, x);
                                } else {
                                    if cur_op == 0 && size > 0 {
                                        // thread_v.insertat(r() % size, x);
                                    } else if cur_op == 1 && size > 0 {
                                        thread_v.at(i, r() % size);
                                    } else if cur_op == 2 && size > 0 {
                                        let pos = r() % size;
                                        match thread_v.at(i, pos) {
                                            Some(_) => {
                                                // thread_v.cwrite(pos, old, x);
                                            },
                                            None => {},
                                        };
                                    }
                                }
                            }
                        }
                    )
                    );
                }
                else if t == erase {
                    threads.push(
                        thread::spawn(
                            move || {
                                let mut rng = rand::thread_rng();
                                let mut r = || -> usize {
                                    rng.gen()
                                };
                                //let tot_ops = thread_ops_per_thread.at(i).unwrap();

                                //TODO: Check why None value
                                let tot_ops = thread_ops_per_thread[i];

                                for _ in 0..tot_ops {
                                    let cur_op = r() % 3;
                                    let do_pushback = (r()%100+100)%100 < limit;

                                    let x = r();
                                    let size = thread_v.length();

                                    if do_pushback {
                                        thread_v.push_back(i, x);
                                    } else {
                                        if cur_op == 0 && size > 0 {
                                            // thread_v.erase(r()%size);
                                            // thread_v.pop_back();
                                        } else if cur_op == 1 && size > 0 {
                                            thread_v.at(i, r()%size);
                                        } else if cur_op == 2 && size > 0 {
                                            let pos = r () % size;
                                            match thread_v.at(i, pos) {
                                                Some(_) => {
                                                    // thread_v.cwrite(pos, old, x);
                                                },
                                                None => {},
                                            };
                                        }
                                    }



                                }


                            }
                        )

                    );
                }
            }

            for t in threads {
                t.join().unwrap();
            }

            let end_time = Instant::now();
            let elapsed_time = end_time.duration_since(start_time);

            print!(",{:?}", elapsed_time.as_millis());
        }
        println!("");
    }



}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_vals_seq(){
        let vec = WaitFreeVector::new(3, 1);
        vec.push_back(0, 10);
        vec.push_back(0, 11);
        vec.push_back(0, 12);
    }
    #[test]
    fn len_seq(){
        let vec = WaitFreeVector::new(3, 1);
        vec.push_back(0, 10);
        vec.push_back(0, 11);
        assert_eq!(vec.length(), 2);
    }

    #[test]
    fn seq_at(){
        let vec = WaitFreeVector::new(2, 1);
        vec.push_back(0, 10);
        vec.push_back(0, 20);
        vec.at(0, 0);

        assert_eq!(vec.at(0, 0), Some(10));
        assert_eq!(vec.at(0, 1), Some(20));
    }

    #[test]
    fn seq_resize_at() {
        // There should be 2 resizes happening here.
        let vec = WaitFreeVector::new(1, 1);

        vec.push_back(0, 10);
        vec.push_back(0, 20);
        vec.push_back(0, 30);
        vec.push_back(0, 40);

        assert_eq!(vec.at(0, 0), Some(10));
        assert_eq!(vec.at(0, 1), Some(20));
        assert_eq!(vec.at(0, 2), Some(30));
        assert_eq!(vec.at(0, 3), Some(40));

        assert_eq!(vec.length(), 4)
    }

    #[test]
    fn threaded_insert_len(){
        let capacity = 100;
        let num_threads = 8;
        let times = 12;
        assert!(num_threads*times < capacity);

        let vec = Arc::new(WaitFreeVector::new(100, num_threads));
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
    fn threaded_insert_and_check_all_are_some(){
        let capacity = 5;
        let num_threads = 4;
        let times = 3;
        // assert!(num_threads*times < capacity);

        let vec = Arc::new(WaitFreeVector::new(capacity, num_threads));
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

        for i in 0..num_threads * times {
            assert!(vec.at(0, i).is_some());
        }
    }

    #[test]
    fn threaded_resize() {
        let capacity = 1;
        let num_threads = 4;
        let times = 5;
        assert!(num_threads*times > capacity);
        
        let vec = Arc::new(WaitFreeVector::new(100, num_threads));
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
        println!("{}", vec.length());
        assert_eq!(vec.length(), num_threads * times);
    }

    #[test]
    fn pop_back() {
        let vec = WaitFreeVector::new(2, 1);
        vec.push_back(0, 10);
        vec.push_back(0, 20);
        vec.at(0, 0);

        assert_eq!(vec.at(0, 0), Some(10));
        assert_eq!(vec.at(0, 1), Some(20));

        assert_eq!(vec.pop_back(0), Some(20));
        assert_eq!(vec.pop_back(0), Some(10));
        assert_eq!(vec.pop_back(0), None);

        assert_eq!(vec.length(), 0);
    }
}

fn main() {
    let num: usize = 3;
    // test_all(num);
    // test_cwrite(num);
    // test_pushback(num);
    // test_popback(num);
}