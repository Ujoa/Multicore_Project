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
    let good = Arc(Vec::with_capacity(num_threads as usize));
    let threads = Vec::new();

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
                        good_thread[i] += v_thread.pop_back();
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
        println!("{}", good[i]);
    }

    println!("/n");
}


fn test_cwrite(num_threads: i32){


}


fn test_all(num_threads: i32){

}

fn main(){
    let v = lib::WaitFreeVector {};
    v.push_back(100);
    v.pop_back();

}