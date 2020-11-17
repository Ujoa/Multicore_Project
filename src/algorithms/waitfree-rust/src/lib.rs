use std::rc::Rc;
use std::sync::Arc;
use crossbeam_epoch::{self as epoch, Atomic, Guard, Shared, Owned};
use std::sync::atomic::Ordering::SeqCst;


// IsDescriptor when non-nul | 0b01
// NotValue when null | 0b00
// NotCopied when null | 0b01
// Resizing when non-null | 0b10

const NotValue: usize = 0b00;
const NotCopied: usize = 0b01;

const MarkDesc: usize = 0b01;
const MarkResize: usize = 0b10;

const TagNotValue: usize = 0;
const TagNotCopied: usize = 1;
const TagDescr: usize = 2;
const TagResize: usize = 3;


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
pub struct WaitFreeVector {
    storage: Atomic<Contiguous>
}


enum PushState {
    Undecided,
    Failed,
    Passed,
}

// replace pushstate enum
const StateUndecided: u8 = 0x00;
const StateFailed: u8 = 0x01;
const StatePassed: u8 = 0x02;



trait DescriptorTrait {
    fn descr_type() -> DescriptorType;
    fn complete(&self, guard: &Guard) -> bool;
    fn value(&self) -> usize;
}

#[derive(Clone)]
enum BaseDescr {
    PushDescrType(PushDescr),
    PopDescrType(PopDescr),
}

// contains the value to be pushed and a state member
#[derive(Clone)]
struct PushDescr {
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
            state: Atomic::new(StateUndecided),
        }
    }

    pub fn statecas(&self, expected: Shared<u8>, new: Owned<u8>, guard: &Guard) {
        self.state.compare_and_set(expected, new, SeqCst, guard);
    }

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
    fn descr_type() -> DescriptorType {
        DescriptorType::PushDescrType
    }
    fn complete(&self, guard: &Guard) -> bool {
        let vectorptr = self.vec.load(SeqCst, guard);
        let vector = unsafe { vectorptr.deref() };
        let spot = vector.get_spot(self.pos, guard);

        if self.pos == 0 {
            self.statecas(StateUndecided, StatePassed);

            spot.compare_and_set(vector.pack_descr(&self), self.value);
        }

        

        
        true
    }
    fn value(&self) -> usize {
        todo!()
    }

    
}

impl WaitFreeVector {
    pub fn length(&self) -> usize{todo!()}
    pub fn get_spot(&self, position: usize, guard: &Guard) -> Atomic<usize> {
        let contigptr = self.storage.load(SeqCst, guard);
        let contig = unsafe { contigptr.deref() };
        let spot = contig.get_spot(position);

        spot
    }

    pub fn pack_descr(descr: Descriptor) {

    }

    pub fn complete_push(&self, spot: Atomic<usize>, shrd: Shared<usize>, descr: &PushDescr, guard: &Guard) {
        let newdescr = descr.clone();
        let mystate = newdescr.state.load(SeqCst, guard);
        let rawstate = unsafe { mystate.deref() }.clone();

        if descr.pos == 0 {
            if rawstate == StateUndecided {
                descr.statecas(mystate, Owned::new(StatePassed), guard)
            }
            
            let newptr = Owned::new(descr).with_tag(TagDescr).into_shared(guard);
            spot.compare_and_set();
        }
    }
}

impl Vector for WaitFreeVector {
    fn push_back(&self, value: usize) -> bool {
        todo!()
    }
    fn pop_back(&self) -> usize { todo!() }
    fn at(&self, _: usize) -> usize { todo!() }
    fn insert_at(&self, _: usize, _: usize) -> bool { todo!() }
    fn erase_at(&self, _: usize) -> bool { todo!() }
    fn cwrite(&self, _: usize, _: usize) -> bool { todo!() }
    //fn announce_op(&self, _: (dyn Descriptor + 'static)) { todo!() }
}

struct Contiguous {
    vector: Atomic<WaitFreeVector>,
    old: Atomic<Contiguous>,
    capacity: usize,
    // array is a regular array of atomic pointers
    array: Vec<Atomic<usize>>,
}

impl Contiguous {
    pub fn new(vector: Atomic<WaitFreeVector>, capacity: usize) -> Contiguous {
        let arr = vec![Atomic::<usize>::null(); capacity];

        // Will use later for NotCopied
        // for i in 0..capacity {
        //     arr[i] = 
        // }

        Contiguous {
            vector,
            old: Atomic::null(),
            capacity,
            array: arr,
        }
    }

    // pub fn new(vector: Box<WaitFreeVector>, old: Box<Contiguous>, capacity: usize) -> Contiguous {
        
    //     let arr = vec![]
    //     Contiguous {
    //         vector,
    //         old,
    //         capacity,

    //     }
    // }

    pub fn get_spot(&self, position: usize) -> Atomic<usize> {
        self.array[position].clone()
    }
}



// PopDescr consists solely of a reference to a PopSubDescr (child) which is initially Null.
#[derive(Clone)]
struct PopDescr {
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
impl PopDescr {
    pub fn new(vec: Rc<Vector>, pos: usize) -> PopDescr {
        PopDescr {
            vec,
            pos,
            child: None
        }
    }
}

impl DescriptorTrait for PopDescr {
    fn descr_type() -> DescriptorType {
        DescriptorType::PopDescrType
    }
    fn complete(&self, guard: &Guard) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}


impl PopSubDescr {
    pub fn new(parent: Rc<PopDescr>, value: usize) -> PopSubDescr {
        PopSubDescr {
            parent,
            value,
        }
    }
}

impl DescriptorTrait for PopSubDescr {
    fn descr_type() -> DescriptorType {
        DescriptorType::PopSubDescrType
    }
    fn complete(&self, guard: &Guard) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}


impl DescriptorTrait for ShiftOp {
    fn descr_type() -> DescriptorType {
        todo!()
    }
    fn complete(&self, guard: &Guard) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl DescriptorTrait for ShiftDescr {
    fn descr_type() -> DescriptorType {
        todo!()
    }
    fn complete(&self, guard: &Guard) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}