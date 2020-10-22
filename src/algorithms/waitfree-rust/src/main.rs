mod lib;
use lib::Vector;
use std::sync::Arc;
use std::thread;



fn test_pushback(num_threads: i32){

    println!("TEST PUSHBACK {} threads", num_threads);
    let v = Arc::new(lib::WaitFreeVector {});
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
        println!("{}", v.at(i));

    }
}

fn test_popback(num_threads: i32){
    let LEN = 30;
    println!("TEST POPBACK {} threads", num_threads);

    let v = Arc::new(lib::WaitFreeVector {});
    let good: Arc<Vec<usize>> = Arc::new(Vec::with_capacity(num_threads as usize));
    let mut threads = Vec::new();

    for i in 0..(num_threads*LEN){
        v.push_back(i as usize);
    }

    for i in 0..num_threads{
        let good_thread = good.clone();
        let v_thread  = v.clone();
        threads.push(
                thread::spawn( move || {
                    for j in 0..LEN{
                        // Needs thread id and first? check :50
                        good_thread[i as usize] += v_thread.pop_back();
                    }

                }
            )
        );
    }

    for t in threads {
        t.join();
    }

    for i in 0..v.length(){
        println!("{}", v.at(i));
    }

    for i in 0..num_threads {
        println!("{}", good[i as usize]);
    }

    println!("/n");
}


fn test_cwrite(num_threads: i32){

    let LEN = 44;

    let v = Arc::new(lib::WaitFreeVector {});
    let mut threads = Vec::new();
    for i in 0..LEN{
        v.push_back(0);
    }

    let cnt = Arc::new(vec![vec![0;LEN];num_threads as usize]);

    for i in 1..num_threads {
        let thread_v = v.clone();
        let thread_cnt = cnt.clone();
        threads.push(
            thread::spawn( move || {

                for j in 0..1000 {
                    let pos = j % thread_v.length();
                    let prev = thread_v.at(pos);

                    if thread_v.cwrite(prev, prev+1) {

                        thread_cnt[i as usize][pos] += 1;
                    }
                }
                }
            )
        );
    }

    for t in threads {
        t.join();
    }

    let tot: Vec<usize> = Vec::with_capacity(LEN);

    for i in 0..num_threads {
        for j in 0..LEN {
            tot[j] += cnt[i as usize][j];
            println!("{} ", cnt[i as usize][j])
        }
        println!();
    }
    println!("-------------");

    for i in 0..LEN{
        println!("{}", v.at(i));
    }

    println!();




}


fn test_all(max_num_threads: usize){

    let max_ops = 6400;
    let insert = 0;
    let erase = 1;
    let limit = 25;

}

fn main(){
    let v = lib::WaitFreeVector {};
    v.push_back(100);
    v.pop_back();

}