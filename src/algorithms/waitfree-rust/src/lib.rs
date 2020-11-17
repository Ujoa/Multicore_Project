use std::rc::Rc;
use std::sync::Arc;
use crossbeam_epoch::{self as epoch, Atomic};
use std::sync::atomic::Ordering::SeqCst;

const NotValue: usize = 0b00;
const NotCopied: usize = 0b01;


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
pub struct WaitFreeVector {}

impl WaitFreeVector {
    pub fn length(&self) -> usize{todo!()}
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

trait Descriptor {
    fn descr_type() -> DescriptorType;
    fn complete(&self) -> bool;
    fn value(&self) -> usize;
}

// PopDescr consists solely of a reference to a PopSubDescr (child) which is initially Null.
struct PopDescr {
    vec: Rc<Vector>,
    pos: usize,
    child: Option<Arc<PopSubDescr>>
}

// PopSubDescr consists of a reference to a previously placed PopDescr (parent)
// and the value that was replaced by the PopSubDescr (value).
struct PopSubDescr {
    parent: Rc<PopDescr>,
    value: usize,
}

enum DescriptorType {
    PushDescrType,
    PopDescrType,
    PopSubDescrType,
}





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

impl Descriptor for PopDescr {
    fn descr_type() -> DescriptorType {
        DescriptorType::PopDescrType
    }
    fn complete(&self) -> bool {
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

impl Descriptor for PopSubDescr {
    fn descr_type() -> DescriptorType {
        DescriptorType::PopSubDescrType
    }
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

enum PushState {
    Undecided,
    Failed,
    Passed,
}

// replace pushstate
const StateUndecided: u8 = 0x00;
const StateFailed: u8 = 0x01;
const StatePassed: u8 = 0x02;

// contains the value to be pushed and a state member
struct PushDescr {
    vec: Atomic<WaitFreeVector>,
    value: usize,
    pos: usize,
    state: PushState
}

impl PushDescr {
    pub fn new(vec: Atomic<WaitFreeVector>, pos: usize, value: usize) -> PushDescr {
        PushDescr {
            vec,
            pos,
            value,
            state: PushState::Undecided,
        }
    }
}

impl Descriptor for PushDescr {
    fn descr_type() -> DescriptorType {
        DescriptorType::PushDescrType
    }
    fn complete(&self) -> bool {
        if self.pos == 0 {

        }
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for ShiftOp {
    fn descr_type() -> DescriptorType {
        todo!()
    }
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for ShiftDescr {
    fn descr_type() -> DescriptorType {
        todo!()
    }
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}