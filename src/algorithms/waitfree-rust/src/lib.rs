use std::rc::Rc;
use std::sync::Arc;

pub trait Vector {
    // API Methods
    fn push_back(&self, value: usize) -> bool;
    fn pop_back(&self) -> usize;
    fn at(&self, index: usize) -> usize;
    fn insert_at(&self, index: usize, element: usize) -> bool;
    fn erase_at(&self, index: usize, element: usize) -> bool;
    fn cwrite(&self, index: usize, element: usize) -> bool;

    // A private method that will be used internally, but
    // not exposed.
    // fn announce_op(&self, descriptor: dyn Descriptor);
}

trait Descriptor {
    fn complete(&self) -> bool;
    fn value(&self) -> usize;
}

// PopDescr consists solely of a reference to a PopSubDescr (child) which is initially Null.
struct PopDescr {
    vec: Rc<Vector>,
    pos: usize,
    child: Arc<PopSubDescr>
}

// PopSubDescr consists of a reference to a previously placed PopDescr (parent)
// and the value that was replaced by the PopSubDescr (value).
struct PopSubDescr {
    parent: Rc<PopDescr>,
    value: usize,
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
impl Descriptor for PopDescr {
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for PopSubDescr {
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for PushDescr {
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for ShiftOp {
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}

impl Descriptor for ShiftDescr {
    fn complete(&self) -> bool {
        todo!()
    }
    fn value(&self) -> usize {
        todo!()
    }
}
