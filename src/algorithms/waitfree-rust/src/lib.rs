use std::rc::Rc;
use std::sync::Arc;
use crossbeam_epoch::{self as epoch, Atomic, Guard, Shared, Owned};
use std::sync::atomic::Ordering::{SeqCst, Release, Acquire};
use std::sync::atomic::{AtomicUsize, AtomicBool};

const TagNotValue: usize = 1;
const TagNotCopied: usize = 2;
const TagDescr: usize = 3;
const TagResize: usize = 4;

const NO_RESULT: usize = usize::MAX;

const LIMIT: usize = usize::MAX;

type Spot = Arc<Atomic<usize>>;

fn make_spot(u: Shared<usize>) -> Spot {
    return Arc::new(Atomic::from(u));
}

type OpSpot = Arc<Atomic<BaseOp>>;

fn make_op_spot(u: Shared<BaseOp>) -> OpSpot {
    return Arc::new(Atomic::from(u));
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
    owner: Atomic<BaseOp>,
    value: usize,
    pos: usize,
    state: Atomic<u8>,
}

impl PushDescr {
    pub fn new(pos: usize, value: usize) -> PushDescr {
        PushDescr {
            owner: Atomic::null(),
            // vec,
            pos,
            value,
            state: Atomic::new(STATE_UNDECIDED),
        }
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

// returns None if the value is NotValue
pub fn value_base(descr: BaseDescr) -> Option<usize> {
    match descr {
        BaseDescr::PushDescrType(d) => Some(d.value),
        _ => None,
    }
}

pub enum BaseOp {
    PushOpType(PushOp),
    PopOpType(PopOp),
}

pub struct PushOp {
    done: Atomic<AtomicBool>,
    value: usize,
    result: Atomic<AtomicUsize>,
    can_return: Atomic<AtomicBool>,
}
impl PushOp {
    pub fn new(value: usize) -> PushOp {
        PushOp {
            done: Atomic::new(AtomicBool::new(false)),
            value,
            result: Atomic::new(AtomicUsize::new(NO_RESULT)),
            can_return: Atomic::new(AtomicBool::new(false)),
        }
    }
}

pub struct PopOp {

}

pub struct WaitFreeVector {
    storage: Atomic<Contiguous>,
    size: Atomic<AtomicUsize>,

    thread_ops: Vec<OpSpot>,
    thread_to_help: Vec<AtomicUsize>,
    num_threads: usize,
}

impl WaitFreeVector {
    pub fn new(capacity: usize, num_threads: usize) -> WaitFreeVector {
        let mut thread_ops: Vec<OpSpot> = Vec::new();
        let mut thread_to_help: Vec<AtomicUsize> = Vec::new();
        // let thread_to_help = vec![0; num_threads];

        for _ in 0..num_threads {
            let i: Shared<BaseOp> = Shared::null();
            thread_ops.push(make_op_spot(i));

            let i: AtomicUsize = AtomicUsize::new(0);
            thread_to_help.push(i);
        }

        WaitFreeVector{
            storage: Atomic::new(Contiguous::new(capacity)),
            size: Atomic::new(AtomicUsize::new(0)),

            thread_ops,
            thread_to_help,
            num_threads,
        }
    }

    pub fn length(&self) -> usize{
        let guard = &epoch::pin();
        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        sizeusizeptr.load(SeqCst)
    }

    pub fn help_if_needed(&self, tid: usize) {
        if tid >= self.num_threads {
            panic!("tid {} is out of bounds", tid);
        }

        let help = self.thread_to_help[tid].load(Acquire);

        self.thread_to_help[tid].store((help + 1) % self.num_threads, Release);

        self.help(tid, help);
    }
    
    pub fn help(&self, mytid: usize, help: usize) {
        let guard = &epoch::pin();

        let opptr = self.thread_ops[help].load(SeqCst, guard);

        if opptr.is_null() {
            return;
        }

        self.an_complete_base(mytid, opptr, guard);

        self.thread_ops[help].compare_and_set(opptr, Shared::null(), SeqCst, guard);
    }

    pub fn get_spot(&self, position: usize, guard: &Guard) -> Spot {
        let contigptr = self.storage.load(SeqCst, guard);
        let contig = unsafe { contigptr.deref() };

        if position >= contig.capacity {
            self.resize();
            return self.get_spot(position, guard);
        }
        let spot = contig.get_spot(position, guard);

        spot
    }

    pub fn resize(&self){
        println!("Resizing");
        let guard = &epoch::pin();
        let old = self.storage.load(SeqCst, guard);

        let mut prefix = 0;
        if !old.is_null() {
            let old = unsafe {old.deref()};
            prefix = old.capacity;
        }

        let new_capacity = prefix * 2 + 1;

        let mut arr: Vec<Spot> = Vec::with_capacity(new_capacity);
        for i in 0..new_capacity {
            if i < prefix {
                let init: Shared<usize> = Shared::null().with_tag(TagNotCopied);
                arr.push(Arc::new(Atomic::from(init)));
            }
            else {
                let init: Shared<usize> = Shared::null().with_tag(TagNotValue);
                arr.push(Arc::new(Atomic::from(init)));
            }
        }

        let old_atomic = self.storage.clone();

        let v_new = Contiguous{
            old: old_atomic,
            capacity: new_capacity,
            array: Atomic::new(arr),
        };

        let shared_cont = self.storage.load(SeqCst, guard);

        match self.storage.compare_and_set(
            shared_cont, Owned::new(v_new), SeqCst,guard) {
            Ok(_) => {
                let shared_newv = self.storage.load(SeqCst, guard);
                let newv = unsafe {shared_newv.deref()};
                for i in 0..new_capacity{
                    newv.copy_value(i, guard);
                }
                // println!("Resize {} ", new_capacity);
            },
            Err(_) => {
                println!("Resize Failed");
            },
        }



    }

    pub fn complete_base(&self, spot: Spot, old: Shared<usize>, descr: &BaseDescr, guard: &Guard) -> bool {
        // let cdescr = descr.clone();
        match descr {
            BaseDescr::PushDescrType(d) => self.complete_push(spot, old, d, guard),
            _ => false,
        }
    }

    pub fn an_complete_base(&self, tid: usize, opptr: Shared<BaseOp>, guard: &Guard) -> bool {
        let op: &BaseOp = unsafe { opptr.deref() };        
        match op {
            BaseOp::PushOpType(o) => self.an_complete_push(tid, o, opptr, guard),
            _ => false,
        }
    }

    // the an_ prefix means this method is to complete an op on the announcement table, not in a descriptor
    pub fn an_complete_push(&self, tid: usize, op: &PushOp, opptr: Shared<BaseOp>, guard: &Guard) -> bool {
        let shsize = self.size.load(SeqCst, guard);
        let usizeptr = unsafe { shsize.deref() }.clone();
        let mut pos = usizeptr.load(SeqCst);

        loop {
            let spot = self.get_spot(pos, guard);
            let expected = spot.load(SeqCst, guard);

            let doneptr = op.done.load(SeqCst, guard);
            let done = unsafe { doneptr.deref() };
            let rawdone = done.load(SeqCst);

            if rawdone {
                break;
            }

            // let expected = spot.load(SeqCst, guard);

            if let Some(x) = unpack_descr(expected, guard) {
                let base = unsafe { x.deref() };
                self.complete_base(spot, expected, base, guard);
                continue;
            }

            if expected.tag() != TagNotValue {
                pos += 1;
                continue;
            }

            let pdescr = PushDescr::new(pos, op.value);
            pdescr.owner.store(opptr, SeqCst);
            let descr = BaseDescr::PushDescrType(pdescr);
            let descrptr = pack_descr(descr.clone(), guard);

            if let Ok(_) = spot.compare_and_set(expected, descrptr, SeqCst, guard) {
                let completeres = self.complete_base(spot, descrptr, &descr, guard);
                
                if completeres {
                    let resptr = op.result.load(SeqCst, guard);
                    let res = unsafe { resptr.deref() };
                    res.store(pos, SeqCst);

                    usizeptr.fetch_add(1, SeqCst);

                    let dptr = op.done.load(SeqCst, guard);
                    let done = unsafe { dptr.deref() };
                    done.store(true, SeqCst);

                    let retptr = op.can_return.load(SeqCst, guard);
                    let ret = unsafe { retptr.deref() };
                    ret.store(true, SeqCst);
                }
                else {
                    if pos == 0 {
                        pos += 1;
                    }
                    else {
                        pos -= 1;
                    }
                }
            }
            // self.get_spot(pos, guard: &Guard)
        }

        loop {
            let retptr = op.can_return.load(SeqCst, guard);
            let ret = unsafe { retptr.deref() };
            if ret.load(SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(3));
        }

        true
    }

    pub fn at(&self, tid: usize, pos: usize) -> Option<usize> {
        let guard = &epoch::pin();

        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        let size = sizeusizeptr.load(SeqCst);

        if pos < size {
            let slot = self.get_spot(pos, guard);
            let ptr = slot.load(SeqCst, guard);

            if ptr.tag() == TagNotValue {
                return None;
            }

            match unpack_descr(ptr, guard) {
                Some(x) => {
                    let descval = unsafe { x.deref() }.clone();
                    return value_base(descval);
                },
                None => {
                    return Some(unsafe { ptr.deref() }.clone());
                },
            }
        }

        None
    }

    pub fn push_back(&self, tid: usize, value: usize) -> usize {
        let guard = &epoch::pin();

        // TODO: announcement table

        let shvalue = Owned::new(value).into_shared(guard);

        if shvalue.is_null() {
            panic!("CANNOT PUSH NULL POINTER");
        }

        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        let mut pos = sizeusizeptr.load(SeqCst);

        for failures in 0..=LIMIT {
            let spot = self.get_spot(pos, guard);
            let expectedptr = spot.load(SeqCst, guard);
            if expectedptr.tag() == TagNotValue 
            // || expectedptr.tag() == TagNotCopied 
            {
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
                // dbg!(expectedptr);
                match unpack_descr(expectedptr, guard) {
                    Some(x) => {
                        let descr = unsafe { x.deref() }.clone();
                        self.complete_base(spot, expectedptr, &descr, guard);
                    }
                    None => {
                        pos += 1;
                    }
                }
            }
            // let expected: usize = unsafe { spotptr.deref() }.clone();
        }

        // TODO: add this op to annoucement table

        0
    }

    pub fn complete_push(&self, spot: Spot, old: Shared<usize>, descr: &PushDescr, guard: &Guard) -> bool {

        let newdescr: PushDescr = descr.clone();

        let (mut mystate, mut rawstate) = loadstate(&newdescr, guard);

        if newdescr.pos == 0 {
            if rawstate == STATE_UNDECIDED {
                descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
            }

            let basedescr = BaseDescr::PushDescrType(newdescr);
            let maskdescr = pack_descr(basedescr, guard);

            // NOTE: Do we need to check if this works or not?
            spot.compare_and_set(old, maskdescr, SeqCst, guard);

            return true;
        }

        let spot2: Spot = self.get_spot(newdescr.pos - 1, guard);
        let current: Shared<usize> = spot2.load(SeqCst, guard);

        let mut failures: usize = 0;

        while rawstate == STATE_UNDECIDED {
            let temp = loadstate(&newdescr, guard);
            mystate = temp.0;
            rawstate = temp.1;

            let spot2: Spot = self.get_spot(newdescr.pos - 1, guard);
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
                let set_to_passed = descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
                if set_to_passed.is_err() {
                    dbg!("Could not update the descriptor state to PASSED");
                }
            }

            self.complete_base(spot2, current, &basedescr, guard);
        }

        let (mystate, rawstate) = loadstate(&newdescr, guard);

        // Descriptor moved out of the way, but we still have to finish this push
        if rawstate == STATE_UNDECIDED {
            if current.tag() == TagNotValue {
                let set_to_failed = descr.state.compare_and_set(mystate, Owned::new(STATE_FAILED), SeqCst, guard);
                if set_to_failed.is_err() {
                    dbg!("Could not update the descriptor state to FAILED");
                }
            }
            else {
                let set_to_passed = descr.state.compare_and_set(mystate, Owned::new(STATE_PASSED), SeqCst, guard);
                if set_to_passed.is_err() {
                    dbg!("Could not update the descriptor state to PASSED");
                }
            }
        }

        let (_, rawstate) = loadstate(&descr, guard);

        if rawstate == STATE_PASSED {
            spot.compare_and_set(old, Owned::new(newdescr.value), SeqCst, guard);
        }
        else {
            spot.compare_and_set(old, Owned::new(0).with_tag(TagNotValue), SeqCst, guard);
        }

        let (_, rawstate) = loadstate(&descr, guard);

        return rawstate == STATE_PASSED;
    }
}

struct Contiguous {
    // vector: Atomic<WaitFreeVector>,
    old: Atomic<Contiguous>,
    capacity: usize,
    // array is a regular array of atomic pointers
    array: Atomic<Vec<Spot>>,
}

impl Contiguous {
    // pub fn new(vector: Atomic<WaitFreeVector>, capacity: usize) -> Contiguous {
    pub fn new(capacity: usize) -> Contiguous {
        let mut arr = Vec::new();

        for failures in 0..capacity {
            let init: Shared<usize> = Shared::null().with_tag(TagNotValue);
            arr.push(make_spot(init));
        }

        Contiguous {
            old: Atomic::null(),
            capacity,
            array: Atomic::new(arr),
        }
    }




    pub fn copy_value(&self, position: usize, guard: &Guard) {
        // Load the old Contiguous structure to copy from
        let oldptr = self.old.load(SeqCst, guard);
        assert!(!oldptr.is_null(), "If we're in copy_value, our pointer to the old vector must exist");

        // Deref and get the old vector
        let old = unsafe { oldptr.deref() };
        let load_vec = unsafe { old.array.load(SeqCst, guard).deref() };

        if position < load_vec.len() {
            let val = load_vec[position].load(SeqCst, guard);
            if val.tag() == TagNotCopied {
                old.copy_value(position, guard);
            }

            // Copying over the value from the old vector into our current vector
            let our_vector = unsafe { self.array.load(SeqCst, guard).deref() };
            let current_spot = our_vector[position].clone();
            
            let expected_value = Shared::<usize>::null().with_tag(TagNotCopied); 

            let reloaded_old_value = load_vec[position].load(SeqCst, guard);
            let updated_our_vector = current_spot.compare_and_set(expected_value, reloaded_old_value, SeqCst, guard);
            if updated_our_vector.is_err() {
                println!("Couldn't overwrite the spot in our vector");
            }
        }
    }

    pub fn get_spot(&self, position: usize, guard: &Guard) -> Spot {
        let vec = unsafe {self.array.load(SeqCst,guard).deref()};
        let spot = vec[position].load(SeqCst, guard);

        if spot.tag() == TagNotCopied {
            self.copy_value(position, guard);
        }

        vec[position].clone()
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