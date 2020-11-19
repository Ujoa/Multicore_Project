use std::rc::Rc;
use std::sync::Arc;
use crossbeam_epoch::{self as epoch, Atomic, Guard, Shared, Owned};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicUsize;


// IsDescriptor when non-nul | 0b01
// NotValue when null | 0b00
// NotCopied when null | 0b01
// Resizing when non-null | 0b10

// const NotValue: usize = 0b00;
// const NotCopied: usize = 0b01;

// const MarkDesc: usize = 0b01;
// const MarkResize: usize = 0b10;

const TagNotValue: usize = 1;
const TagNotCopied: usize = 2;
const TagDescr: usize = 3;
const TagResize: usize = 4;

const LIMIT: usize = usize::MAX;


pub trait Vector {
    // API Methods
    fn push_back(&self, value: usize) -> bool;
    fn pop_back(&self) -> usize;
    fn at(&self, index: usize) -> usize;
    fn insert_at(&self, index: usize, element: usize) -> bool;
    fn erase_at(&self, index: usize) -> bool;
    fn cwrite(&self, index: usize, element: usize) -> bool;

    // A private method that will be used internally, but
    // not exposed.
    // fn announce_op(&self, descriptor: dyn Descriptor);
}



enum PushState {
    Undecided,
    Failed,
    Passed,
}

// replace pushstate enum
const STATE_UNDECIDED: u8 = 0x00;
const STATE_FAILED: u8 = 0x01;
const STATE_PASSED: u8 = 0x02;

trait DescriptorTrait {
    // fn descr_type() -> DescriptorType;
    fn complete(&self, guard: &Guard) -> bool;
    fn value(&self) -> usize;
}

#[derive(Clone)]
pub enum BaseDescr {
    PushDescrType(PushDescr),
    PopDescrType(PopDescr),
}

// contains the value to be pushed and a state member
#[derive(Clone)]
pub struct PushDescr {
    // vec: Atomic<WaitFreeVector>,
    value: usize,
    pos: usize,
    state: Atomic<u8>,
}

impl PushDescr {
    // vec: Atomic<WaitFreeVector>, 
    pub fn new(pos: usize, value: usize) -> PushDescr {
        PushDescr {
            // vec,
            pos,
            value,
            state: Atomic::new(STATE_UNDECIDED),
        }
    }

    // pub fn statecas(&self, expected: Shared<u8>, new: Owned<u8>, guard: &Guard) {
    //     self.state.compare_and_set(expected, new, SeqCst, guard);
    // }

    // pub fn stateload(&self, guard: &Guard) -> (Shared<u8>, &Guard) {
    //     (self.state.load(SeqCst, guard), guard)
    // }

    // pub fn give_me(&self) -> PushDescr {
    //     PushDescr {
    //         value: self.value,

    //     }
    // }
}

impl DescriptorTrait for PushDescr {
    // fn descr_type() -> DescriptorType {
    //     DescriptorType::PushDescrType
    // }
    fn complete(&self, guard: &Guard) -> bool {
        // let vectorptr = self.vec.load(SeqCst, guard);
        // let vector = unsafe { vectorptr.deref() };
        // let spot = vector.get_spot(self.pos, guard);

        // if self.pos == 0 {
        //     self.statecas(StateUndecided, StatePassed);

        //     spot.compare_and_set(vector.pack_descr(&self), self.value);
        // }

        

        
        true
    }
    fn value(&self) -> usize {
        todo!()
    }

    
}

pub fn pack_descr(descr: BaseDescr, guard: &Guard) -> Shared<usize> {
    let ptr = Owned::new(descr).with_tag(TagDescr).into_shared(guard);
    let masked: Shared<usize> = unsafe { std::mem::transmute(ptr) };
    masked
}

pub fn unpack_descr<'g>(curr: Shared<usize>, guard: &'g Guard) -> Option<Shared<'g, BaseDescr>> {
    let unmasked: Shared<BaseDescr> = unsafe { std::mem::transmute(curr) };
    
    if unmasked.tag() == TagDescr {
        Some(unmasked)
    }
    else {
        None
    }
}

pub fn loadstate<'g>(newdescr: &PushDescr, guard: &'g Guard) -> (Shared<'g, u8>, u8) {
    let mystate = newdescr.state.load(SeqCst, guard);
    if mystate.is_null() {
        panic!("STATE OF A DESCRIPTOR WAS NULL")
    }
    let rawstate: u8 = unsafe { mystate.deref() }.clone();

    (mystate, rawstate)
}

pub struct WaitFreeVector {
    storage: Atomic<Contiguous>,
    size: Atomic<AtomicUsize>,
}

impl WaitFreeVector {
    pub fn new(capacity: usize) -> WaitFreeVector {
        WaitFreeVector{
            storage: Atomic::new(Contiguous::new(capacity)),
            size: Atomic::new(AtomicUsize::new(0)),
        }
    }

    pub fn length(&self) -> usize{todo!()}
    pub fn get_spot(&self, position: usize, guard: &Guard) -> Atomic<usize> {
        let contigptr = self.storage.load(SeqCst, guard);
        let contig = unsafe { contigptr.deref() };
        let spot = contig.get_spot(position, guard);

        spot
    }
    
    pub fn complete_base(&self, spot: Atomic<usize>, old: Shared<usize>, descr: &BaseDescr, guard: &Guard) -> bool {
        // let cdescr = descr.clone();
        match descr {
            BaseDescr::PushDescrType(d) => self.complete_push(spot, old, d, guard),
            _ => false,
        }
    }

    pub fn at(&self, tid: usize, pos: usize) -> Option<usize> {
        // let guard = &epoch::pin();

        // let shsize = self.size.load(SeqCst, guard);
        // let sizeusizeptr = unsafe { shsize.deref() }.clone();
        // let size = sizeusizeptr.load(SeqCst);

        // if pos < size

        None
    }

    pub fn push_back(&self, tid: usize, value: usize) -> usize {
        let guard = &epoch::pin();
        
        // TODO: announcement table

        let shvalue = Owned::new(value).into_shared(guard);

        if shvalue.is_null() {
            panic!("CANNOT PUSH NULL POINTER");
        }
        
        // Should be safe, user should never pass us a descriptor
        // let realvalue = unsafe { shvalue.deref() }.clone();

        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        let mut pos = sizeusizeptr.load(SeqCst);

        for failures in 0..=LIMIT {
            let spot = self.get_spot(pos, guard);
            let expectedptr = spot.load(SeqCst, guard);
            if expectedptr.tag() == TagNotValue {
                if pos == 0 {
                    let res = spot.compare_and_set(expectedptr, shvalue, SeqCst, guard);
                    match res {
                        Ok(_) => {
                            sizeusizeptr.fetch_add(1, SeqCst);
                            return 0;
                        },
                        Err(_) => {
                            
                            pos += 1;
                            continue;
                        },
                    }
                }

                let descr = BaseDescr::PushDescrType(PushDescr::new(pos, value));
                let cdescr = descr.clone();
                let descrptr = pack_descr(descr, guard);

                match spot.compare_and_set(expectedptr, descrptr, SeqCst, guard) {
                    Ok(_) => {
                        if self.complete_base(spot, descrptr, &cdescr, guard) {
                            sizeusizeptr.fetch_add(1, SeqCst);
                            return pos;
                        }
                        else {
                            pos -= 1;
                        }
                    },
                    Err(_) => (),
                }
            }
            else {
                dbg!(expectedptr);
                match unpack_descr(expectedptr, guard) {
                    Some(x) => {
                        let descr = unsafe { x.deref() }.clone();
                        self.complete_base(spot, expectedptr, &descr, guard);
                    }
                    None => {
                        println!("&");
                        pos += 1;
                    }
                }
            }
            // let expected: usize = unsafe { spotptr.deref() }.clone();
        }

        // TODO: add this op to annoucement table 

        0
    }

    pub fn complete_push(&self, spot: Atomic<usize>, old: Shared<usize>, descr: &PushDescr, guard: &Guard) -> bool {
        // use WaitFreeVector;

        let newdescr: PushDescr = descr.clone();
        // let mystate: Shared<u8> = newdescr.state.load(SeqCst, guard);
        // if mystate.is_null() {
        //     panic!("STATE OF A DESCRIPTOR WAS NULL IN complete_push")
        // }

        // let rawstate: u8 = unsafe { mystate.deref() }.clone();

        let (mut mystate, mut rawstate) = loadstate(&newdescr, guard);
        
        if newdescr.pos == 0 {
            if rawstate == STATE_UNDECIDED {
                descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
            }

            let basedescr = BaseDescr::PushDescrType(newdescr);
            let maskdescr = pack_descr(basedescr, guard);
            
            spot.compare_and_set(old, maskdescr, SeqCst, guard);

            return true;
        }

        let spot2: Atomic<usize> = self.get_spot(newdescr.pos - 1, guard);
        let current: Shared<usize> = spot2.load(SeqCst, guard);

        let mut failures: usize = 0;

        while rawstate == STATE_UNDECIDED {
            let temp = loadstate(&newdescr, guard);
            mystate = temp.0;
            rawstate = temp.1;

            let spot2: Atomic<usize> = self.get_spot(newdescr.pos - 1, guard);
            let current: Shared<usize> = spot2.load(SeqCst, guard);
            let unpackres = unpack_descr(current, guard);
            match unpackres {
                None => break,
                _ => (),
            }
            let baseptr = unpackres.unwrap();
            let basedescr = unsafe { baseptr.deref() }.clone();

            failures += 1;
            if failures >= LIMIT {
                descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
            }

            self.complete_base(spot2, current, &basedescr, guard);
            
            // Reset for next loop
            // let mystate = newdescr.state.load(SeqCst, guard);
            // if mystate.is_null() {
            //     panic!("STATE OF A DESCRIPTOR WAS NULL IN complete_push")
            // }
            // let rawstate: u8 = unsafe { mystate.deref() }.clone();

            

            // let spot2: Atomic<usize> = self.get_spot(newdescr.pos - 1, guard);
            // let current: Shared<usize> = spot2.load(SeqCst, guard);
        }

        let temp = loadstate(&newdescr, guard);
        mystate = temp.0;
        rawstate = temp.1;

        // Descriptor moved out of the way, but we still have to finish this push
        if rawstate == STATE_UNDECIDED {
            if current.tag() == TagNotValue {
                descr.state.compare_and_set(mystate, Owned::new(STATE_FAILED), SeqCst, guard);
            }
            else {
                descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
            }
        }

        let temp = loadstate(&newdescr, guard);
        mystate = temp.0;
        rawstate = temp.1;

        if rawstate == STATE_PASSED {
            spot.compare_and_set(old, Owned::new(newdescr.value), SeqCst, guard);
        }
        else {
            spot.compare_and_set(old, Owned::new(0).with_tag(TagNotValue), SeqCst, guard);
        }

        let temp = loadstate(&newdescr, guard);
        mystate = temp.0;
        rawstate = temp.1;

        return rawstate == STATE_PASSED;
    }
}

// impl Vector for WaitFreeVector {
//     fn push_back(&self, value: usize) -> bool {
//         todo!()
//     }
//     fn pop_back(&self) -> usize { todo!() }
//     fn at(&self, _: usize) -> usize { todo!() }
//     fn insert_at(&self, _: usize, _: usize) -> bool { todo!() }
//     fn erase_at(&self, _: usize) -> bool { todo!() }
//     fn cwrite(&self, _: usize, _: usize) -> bool { todo!() }
//     //fn announce_op(&self, _: (dyn Descriptor + 'static)) { todo!() }
// }

struct Contiguous {
    // vector: Atomic<WaitFreeVector>,
    old: Atomic<Contiguous>,
    capacity: usize,
    // array is a regular array of atomic pointers
    array: Vec<Atomic<usize>>,
}

impl Contiguous {
    // pub fn new(vector: Atomic<WaitFreeVector>, capacity: usize) -> Contiguous {
    pub fn new(capacity: usize) -> Contiguous {
        let init: Shared<usize> = Shared::null().with_tag(TagNotValue);
        let arr: Vec<Atomic<usize>> = vec![Atomic::from(init); capacity];

        // Will use later for NotCopied
        // for i in 0..capacity {
        //     arr[i] = 
        // }

        Contiguous {
            // vector,
            old: Atomic::null(),
            capacity,
            array: arr,
        }
    }

    pub fn copy_value(&self, position: usize, guard: &Guard) {
        // let oldptr = self.old.load(SeqCst, guard);
        // if !oldptr.is_null() {
        //     let old = unsafe { oldptr.deref() }.clone();
        //     old.array[position]
        // }

        // copy value
        todo!();
    }

    pub fn get_spot(&self, position: usize, guard: &Guard) -> Atomic<usize> {
        if position >= self.capacity {
            // resize
            // dbg!(position);
            // dbg!(self.capacity);
            // todo!();
        }

        let spot = &self.array[position];

        if spot.load(SeqCst, guard).tag() == TagNotCopied {
            self.copy_value(position, guard);
        }

        self.array[position].clone()
    }
}



// PopDescr consists solely of a reference to a PopSubDescr (child) which is initially Null.
#[derive(Clone)]
pub struct PopDescr {
    vec: Atomic<WaitFreeVector>,
    pos: usize,
    child: Atomic<PopSubDescr>
}

// PopSubDescr consists of a reference to a previously placed PopDescr (parent)
// and the value that was replaced by the PopSubDescr (value).
struct PopSubDescr {
    parent: Rc<PopDescr>,
    value: usize,
}

// #[derive(Clone)]
// enum DescriptorType {
//     PushDescrType,
//     PopDescrType,
//     PopSubDescrType,
// }





struct ShiftOp {
    vec: Rc<Vector>,
    pos: usize,
    incomplete: bool,
    next: Arc<ShiftDescr>,
}

struct ShiftDescr {
    op: Rc<ShiftOp>,
    pos: usize,
    value: usize,
    prev: Rc<ShiftDescr>,
    next: Arc<ShiftDescr>,
}

// Implementations for the different Descriptors
// impl PopDescr {
//     pub fn new(vec: Rc<Vector>, pos: usize) -> PopDescr {
//         PopDescr {
//             vec,
//             pos,
//             child: None
//         }
//     }
// }

// impl DescriptorTrait for PopDescr {
//     fn descr_type() -> DescriptorType {
//         DescriptorType::PopDescrType
//     }
//     fn complete(&self, guard: &Guard) -> bool {
//         todo!()
//     }
//     fn value(&self) -> usize {
//         todo!()
//     }
// }


// impl PopSubDescr {
//     pub fn new(parent: Rc<PopDescr>, value: usize) -> PopSubDescr {
//         PopSubDescr {
//             parent,
//             value,
//         }
//     }
// }

// impl DescriptorTrait for PopSubDescr {
//     fn descr_type() -> DescriptorType {
//         DescriptorType::PopSubDescrType
//     }
//     fn complete(&self, guard: &Guard) -> bool {
//         todo!()
//     }
//     fn value(&self) -> usize {
//         todo!()
//     }
// }


// impl DescriptorTrait for ShiftOp {
//     fn descr_type() -> DescriptorType {
//         todo!()
//     }
//     fn complete(&self, guard: &Guard) -> bool {
//         todo!()
//     }
//     fn value(&self) -> usize {
//         todo!()
//     }
// }

// impl DescriptorTrait for ShiftDescr {
//     fn descr_type() -> DescriptorType {
//         todo!()
//     }
//     fn complete(&self, guard: &Guard) -> bool {
//         todo!()
//     }
//     fn value(&self) -> usize {
//         todo!()
//     }
// }