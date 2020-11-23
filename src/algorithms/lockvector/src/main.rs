mod lib;
use lib::LockVector;


use std::sync::Arc;
use std::thread;
use rand::Rng;
use std::time::Instant;


fn test_pushback(num_threads: usize){

    println!("TEST PUSHBACK {} threads", num_threads);
    // let size = (num_threads as usize) * 30;
    let size = num_threads;
    let v = Arc::new(LockVector::new(size));
    let mut threads = Vec::new();


    for i in 0..num_threads{
        let thread_v = v.clone();
        threads.push(
            thread::spawn(move || {

                for j in 0..30{
                    thread_v.push_back(((i as usize)+1)*100+(j as usize));
                }
                
            })
        );
    }

    for t in threads{
        t.join().unwrap();
    }

    for i in 0..v.length(){
        println!("{}", v.at(0).unwrap());
    }
}

fn test_popback(num_threads: usize){
    let len = 30;
    let size = num_threads;
    // let size = 1000;
    println!("TEST POPBACK {} threads", num_threads);

    let v = Arc::new(LockVector::new(size));
    // let good: Arc<WaitFreeVector> = Arc::new(WaitFreeVector::new(num_threads as usize));
    let mut threads = Vec::new();

    for i in 0..size{
        v.push_back(i as usize);
    }

    for i in 0..num_threads{
        let v_thread  = v.clone();

        
        threads.push(
                thread::spawn( move || {
                    let mut res = 0;
                    for _ in 0..len{
                        // let val = 10;
                        if let Some(val) = v_thread.pop_back() {
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
        println!("{}", v.at(i).unwrap());
    }

    if v.length() > 0 {
        match v.pop_back() {
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
    let size = num_threads;

    let v = Arc::new(LockVector::new(size));
    let mut threads = Vec::new();
    for _ in 0..len{
        v.push_back(0);
    }

    let mut cnt = Vec::new();
    for _ in 0..num_threads{
        let new_v = Arc::new(LockVector::new(len));
        for _ in 0..len {
            new_v.push_back(0);
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
                    let prev = thread_v.at(pos).unwrap();

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
            let val = cnt[i as usize].at(j).unwrap();
            tot[j] += val;
            println!("{} ", val);
        }
        println!();
    }
    println!("-------------");

    for i in 0..len{
        println!("{}", v.at(i).unwrap());
    }

    println!();

}


fn test_all(max_num_threads: usize){

    let max_ops = 128000;
    let insert = 0;
    let erase = 1;
    let limit = 25;

    for num_threads in 1..max_num_threads+1 {

        print!("{}",num_threads);

        for t in vec![insert, erase]{
            let v = Arc::new(LockVector::new(num_threads+1));

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
                v.push_back(i);
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
                                    thread_v.push_back(x);
                                } else {
                                    if cur_op == 0 && size > 0 {
                                        // thread_v.insertat(r() % size, x);
                                    } else if cur_op == 1 && size > 0 {
                                        thread_v.at(r() % size);
                                    } else if cur_op == 2 && size > 0 {
                                        let pos = r() % size;
                                        match thread_v.at(pos) {
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
                                        thread_v.push_back(x);
                                    } else {
                                        if cur_op == 0 && size > 0 {
                                            // thread_v.erase(r()%size);
                                            // thread_v.pop_back();
                                        } else if cur_op == 1 && size > 0 {
                                            thread_v.at( r()%size);
                                        } else if cur_op == 2 && size > 0 {
                                            let pos = r () % size;
                                            match thread_v.at(pos) {
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


fn main() {
    let num: usize = 64;
    test_all(num);
    // test_cwrite(num);
    // test_pushback(num);
    // test_popback(num);
}