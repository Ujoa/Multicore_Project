#![allow(unused)]
use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};
use std::sync::atomic::Ordering::SeqCst;

#[derive(Clone)]
struct PushDescr {
    // vec: Atomic<WaitFreeVector>,
    value: usize,
    pos: usize,
    state: Atomic<u8>,
}

const StateUndecided: u8 = 0x00;
const StateFailed: u8 = 0x01;
const StatePassed: u8 = 0x02;

const MarkDesc: usize = 0b01;
const MarkResize: usize = 0b10;

const TagNotValue: usize = 0;
const TagNotCopied: usize = 1;
const TagDescr: usize = 2;
const TagResize: usize = 3;

fn main() {
    println!("{}", (1 << std::mem::align_of::<usize>().trailing_zeros()));

    let guard = &epoch::pin();

    let descr = PushDescr {
        value: 100, 
        pos: 4, 
        state: Atomic::new(StateUndecided),
    };
    let ptr = Owned::new(descr).with_tag(TagDescr).into_shared(guard);

    let atm = Atomic::<usize>::new(1000);

    // let b = ptr.load(SeqCst, guard);
    let sh = atm.load(SeqCst, guard);
    println!("ptr-{:?}", ptr.as_raw());
    println!("rtr-{:?}", sh.as_raw());

    let masked: Shared<usize> = unsafe { std::mem::transmute(ptr) };

    // let test = Owned::new(descr);

    atm.compare_and_set(sh, masked, SeqCst, guard);
    let sh2 = atm.load(SeqCst, guard);
    println!("fin-{:?}", sh2.as_raw());
    println!("tag-{:?}", sh2.tag());



    // for i in 0..10 {
    //     let test = Atomic::new(1234);
    //     let test1 = Atomic::<usize>::null();
        
        
    //     let test3 = test2[0].load(SeqCst, guard);
    //     let b = test.load(SeqCst, guard);
    //     let a = test3.as_raw();

    //     for j in test2 {
    //         let curr = test.load(SeqCst, guard);
    //         println!("{:?}", curr.as_raw());
    //     }
    
        
    // }
    
    
    // let p = a.load(SeqCst, guard);
    
    // test2[0}
}