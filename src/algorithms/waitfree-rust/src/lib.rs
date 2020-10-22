use std::rc::Rc;
use std::sync::Arc;


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
    
    fn push_back(&self, _: usize) -> bool { todo!() }
    fn pop_back(&self) -> usize { todo!() }
    fn at(&self, _: usize) -> usize { todo!() }
    fn insert_at(&self, _: usize, _: usize) -> bool { todo!() }
    fn erase_at(&self, _: usize) -> bool { todo!() }
    fn cwrite(&self, _: usize, _: usize) -> bool { todo!() }
    //fn announce_op(&self, _: (dyn Descriptor + 'static)) { todo!() }
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

enum PushState {
    Undecided,
    Failed,
    Passed,
}

// contains the value to be pushed and a state member
struct PushDescr {
    vec: Rc<Vector>,
    value: usize,
    pos: usize,
    state: PushState
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


impl PushDescr {
    pub fn new(vec: Rc<Vector>, pos: usize, value: usize) -> PushDescr {
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
        todo!()
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