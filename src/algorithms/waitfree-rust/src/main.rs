mod lib;
use lib::WaitFreeVector;
use std::sync::Arc;
use std::thread;
use rand::Rng;
use std::time::{Instant, Duration};


// fn test_pushback(num_threads: i32){

//     println!("TEST PUSHBACK {} threads", num_threads);
//     let v = Arc::new(lib::WaitFreeVector {});
//     let mut threads = Vec::new();

//     for i in 0..num_threads{
//         let thread_v = v.clone();
//         threads.push(
//             thread::spawn(move || {

//                 for j in 0..30{
//                     thread_v.push_back(((i as usize)+1)*100+(j as usize));
//                 }
                
//             })
//         );
//     }

//     for t in threads{
//         t.join().unwrap();
//     }

//     for i in 0..v.length(){
//         println!("{}", v.at(i));

//     }
// }

// fn test_popback(num_threads: i32){
//     let LEN = 30;
//     println!("TEST POPBACK {} threads", num_threads);

//     let v = Arc::new(lib::WaitFreeVector {});
//     let good: Arc<Vec<usize>> = Arc::new(Vec::with_capacity(num_threads as usize));
//     let mut threads = Vec::new();

//     for i in 0..(num_threads*LEN){
//         v.push_back(i as usize);
//     }

//     for i in 0..num_threads{
//         let good_thread = good.clone();
//         let v_thread  = v.clone();
//         threads.push(
//                 thread::spawn( move || {
//                     for j in 0..LEN{
//                         // Needs thread id and first? check :50
//                         good_thread[i as usize] += v_thread.pop_back();
//                     }

//                 }
//             )
//         );
//     }

//     for t in threads {
//         t.join();
//     }

//     for i in 0..v.length(){
//         println!("{}", v.at(i));
//     }

//     for i in 0..num_threads {
//         println!("{}", good[i as usize]);
//     }

//     println!("/n");
// }


// fn test_cwrite(num_threads: i32){

//     let LEN = 44;

//     let v = Arc::new(lib::WaitFreeVector {});
//     let mut threads = Vec::new();
//     for i in 0..LEN{
//         v.push_back(0);
//     }

//     let cnt = Arc::new(vec![vec![0;LEN];num_threads as usize]);

//     for i in 1..num_threads {
//         let thread_v = v.clone();
//         let thread_cnt = cnt.clone();
//         threads.push(
//             thread::spawn( move || {

//                 for j in 0..1000 {
//                     let pos = j % thread_v.length();
//                     let prev = thread_v.at(pos);

//                     if thread_v.cwrite(prev, prev+1) {

//                         thread_cnt[i as usize][pos] += 1;
//                     }
//                 }
//                 }
//             )
//         );
//     }

//     for t in threads {
//         t.join();
//     }

//     let mut tot: Vec<usize> = Vec::with_capacity(LEN);

//     for i in 0..num_threads {
//         for j in 0..LEN {
//             tot[j] += cnt[i as usize][j];
//             println!("{} ", cnt[i as usize][j])
//         }
//         println!();
//     }
//     println!("-------------");

//     for i in 0..LEN{
//         println!("{}", v.at(i));
//     }

//     println!();

// }


// fn test_all(max_num_threads: usize){

//     let max_ops = 6400;
//     let insert = 0;
//     let erase = 1;
//     let limit = 25;

//     for num_threads in 1..max_num_threads {

//         println!("{}",num_threads);

//         for t in vec![insert, erase]{
//             let v = Arc::new(lib::WaitFreeVector {});

//             let each_thread = max_ops/num_threads;
//             let extra = max_ops % num_threads;
//             let ops_per_thread = Arc::new(lib::WaitFreeVector {});

//             for i in 1..num_threads {
//                 ops_per_thread.insert_at(i, each_thread);
//                 if i <= extra{
//                     ops_per_thread.insert_at(i, ops_per_thread.at(i)+1);
//                 }
//             }

//             let start_time = Instant::now();
//             for i in 0..10 {
//                 v.push_back(i);
//             }

//             let mut threads = Vec::new();

//             for i in 1..num_threads{
//                 let thread_ops_per_thread = ops_per_thread.clone();
//                 let thread_v = v.clone();

//                 if t == insert {
//                     threads.push(
//                         thread::spawn(
//                         move || {
//                             let mut rng = rand::thread_rng();
//                             let mut r = || -> usize {
//                                 rng.gen_range(0, i)
//                             };

//                             let tot_ops = thread_ops_per_thread.at(i);
//                             for j in 0..tot_ops {
//                                 let cur_op = r() % 3;
//                                 let do_pushack = (r()%100+100)%100 < limit;

//                                 let x = r();
//                                 let size = thread_v.length();
//                                 if do_pushack {
//                                     thread_v.push_back(x);
//                                 } else {
//                                     if cur_op == 0 && size > 0 {
//                                         thread_v.insert_at(r() % size, x);
//                                     } else if cur_op == 1 && size > 0 {
//                                         thread_v.at(r() % size);
//                                     } else if cur_op == 2 && size > 0 {
//                                         let pos = r() % size();
//                                         let old = thread_v.at(pos);
//                                         thread_v.cwrite(pos, old);
//                                     }
//                                 }
//                             }
//                         }
//                     )
//                     );
//                 }
//                 else if t == erase {
//                     threads.push(
//                         thread::spawn(
//                             move || {
//                                 let mut rng = rand::thread_rng();
//                                 let mut r = || -> usize {
//                                     rng.gen_range(0, i)
//                                 };
//                                 let tot_ops = thread_ops_per_thread.at(i);
//                                 for j in 0..tot_ops {
//                                     let cur_op = r() % 3;
//                                     let do_pushback = (r()%100+100)%100 < limit;

//                                     let x = r();
//                                     let size = thread_v.length();

//                                     if do_pushback {
//                                         thread_v.push_back(x);
//                                     } else {
//                                         if cur_op == 0 && size > 0 {
//                                             thread_v.erase_at(r()%size);
//                                         } else if cur_op == 1 && size > 0 {
//                                             thread_v.at(r()%size);
//                                         } else if cur_op == 2 && size > 0 {
//                                             let pos = r () % size;
//                                             let old = thread_v.at(pos);
//                                             thread_v.cwrite(pos, old);
//                                         }
//                                     }



//                                 }


//                             }
//                         )

//                     );
//                 }
//             }

//             for t in threads {
//                 t.join();
//             }

//             let end_time = Instant::now();
//             let elapsed_time = end_time.duration_since(start_time);

//             println!(", {:?}", elapsed_time);
//         }
//         println!("");
//     }



// }

fn main(){
    // let num: i32 = 10;
    // test_all(num as usize);
    // test_cwrite(num);
    // test_pushback(num);
    // test_popback(num);

    // let capacity = 100;
    // let num_threads = 8;
    // assert!(num_threads < capacity);
    
    // let vec = Arc::new(WaitFreeVector::new(100));
    // let mut handles = Vec::new();
    
    // for i in 0..num_threads {

    //     let vec_thread = vec.clone();
    //     handles.push(
    //         thread::spawn(
    //             move || {
    //                 for _ in 0..2 {
    //                     vec_thread.push_back(i, i*10);
    //                 }

    //                 vec_thread.at(0, i)
    //             }
    //         )
    //     );
    // }

    // for handle in handles {
    //     handle.join().unwrap();
    // }

    let vec = WaitFreeVector::new(100);

    vec.push_back(0, 10);
    vec.push_back(0, 11);
    vec.push_back(0, 12);
    dbg!(vec.at(0, 2));
}