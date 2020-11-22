use std::rc::Rc;
use std::sync::Arc;
use crossbeam_epoch::{self as epoch, Atomic, Guard, Shared, Owned};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicUsize;

const TagNotValue: usize = 1;
const TagNotCopied: usize = 2;
const TagDescr: usize = 3;
const TagResize: usize = 4;

const LIMIT: usize = usize::MAX;

type Spot = Arc<Atomic<usize>>;

fn make_spot(u: Shared<usize>) -> Spot {
    return Arc::new(Atomic::from(u));
}

// replace pushstate enum
const STATE_UNDECIDED: u8 = 0x00;
const STATE_FAILED: u8 = 0x01;
const STATE_PASSED: u8 = 0x02;

#[derive(Clone)]
pub enum BaseDescr {
    PushDescrType(PushDescr),
    PopDescrType(PopDescr),
    PopSubDescrType(PopSubDescr),
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
    pub fn new(pos: usize, value: usize) -> PushDescr {
        PushDescr {
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

pub fn is_descr(curr: Shared<usize>, guard: &'g Guard) -> bool {
    let unmasked: Shared<BaseDescr> = unsafe { std::mem::transmute(curr) };
    
    if unmasked.tag() == TagDescr && unmasked != TagDescr {
        return true;
    }
    else {
        return false;
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

pub fn value_base(descr: BaseDescr) -> Option<usize> {
    match descr {
        BaseDescr::PushDescrType(d) => Some(d.value),
        BaseDescr::PopDescrType(d) => Some(d.value),
        BaseDescr::PopSubDescrType(d) => Some(d.value),
        _ => None,
    }
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

    pub fn length(&self) -> usize{
        let guard = &epoch::pin();
        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        sizeusizeptr.load(SeqCst)
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
            BaseDescr::PopDescrType(d) => self.complete_pop(spot, old, d, guard),
            BaseDescr::PopSubDescrType(d) => self.complete_pop_sub(spot, old, d, guard),
            _ => false,
        }
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

    pub fn pop_back(&self, tid: usize) -> Option<Atomic<usize>> {
        let guard = &epoch::pin();
        
        // TODO: announcement table
    
        let shsize = self.size.load(SeqCst, guard);
        let sizeusizeptr = unsafe { shsize.deref() }.clone();
        let mut pos = sizeusizeptr.load(SeqCst);
    
        for failures in 0..=LIMIT {
            if pos == 0 {
                return None;
            }
    
            let spot = self.get_spot(pos, guard);
            let expectedptr = spot.load(SeqCst, guard);
            if expectedptr.tag() == TagNotValue {
                
                let descr = BaseDescr::PopDescrType(PopDescr::new(pos));
                // let cdescr = descr.clone();
                let descrptr = pack_descr(descr, guard);
    
                match spot.compare_and_set(expectedptr, descrptr, SeqCst, guard) {
                    Ok(_) => {
                        let res = self.complete_base(spot, descrptr, &descr, guard);
                        if res {
                            // let newdescr: PopDescr = descr.clone();
                            // let child = descr.child;
                            
                            let mut value = &descr.child.load(SeqCst);
                            
                            sizeusizeptr.fetch_add(-1, SeqCst);
                            return Some(value);
                        }
                        else {
                            pos -= 1;
                        }
                    },
                    Err(_) => (),
    
                }
            }
            else {
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
            
            // announcement table stuff
        }

        None
    }
        
    pub fn complete_pop(&self, spot: Atomic<usize>, old: Shared<usize>, descr: &PopDescr, guard: &Guard) -> bool {
    
        let newdescr = descr.clone();

        let vectorptr = self.vec.load(SeqCst, guard);
        let vector = unsafe { vectorptr.deref() };
        let spot = vector.get_spot(self.pos, guard);

        let child = self.child;
        let mut value = child.load(SeqCst, guard);
        let mut failures: usize = 0;

        let (mut mystate, mut rawstate) = loadstate(&newdescr, guard);

        while !value {
            let temp = loadstate(&newdescr, guard);
            mystate = temp.0;
            rawstate = temp.1;

            failures += 1;
            if failures >= LIMIT {
                child.compare_and_set(mystate, Owned::new(STATE_FAILED), SeqCst, guard);
                break;
            }

            let spot: Atomic<usize> = self.get_spot(newdescr.pos - 1, guard);
            let expected: Shared<usize> = spot.load(SeqCst, guard);
            let unpackres = unpack_descr(expected, guard);
            match unpackres {
                None => break,
                _ => (),
            }
            let baseptr = unpackres.unwrap();
            let basedescr = unsafe { baseptr.deref() }.clone();

            if expected.tag() == TagNotValue {
                child.compare_and_set(mystate, Owned::new(STATE_FAILED), SeqCst, guard);
            }
            else if vector.is_descr(expected) { 
                self.complete_base(spot, expected, &basedescr, guard);
            }
            else {
                let popsubdescr = BaseDescr::PopSubDescrType(PopSubDescr::new(self, expected));
                let packed = vector.pack_descr(popsubdescr);

                if spot.compare_and_set(expected, packed, SeqCst, guard) {
                    let psh_type = Some(popsubdescr);

                    if psh_type {
                        child.compare_and_set(mystate, popsubdescr, SeqCst, guard);
                    } 
                    if value == popsubdescr {
                        spot.compare_and_set(packed, NotValue, SeqCst, guard);
                    }
                    else {
                        let packdescr = vector.packdescr(self);
                        spot.compare_and_set(packdescr, expected, SeqCst, guard);
                    }
                }
            }
        }
        
        let packing = self.vec.pack_descr(self);
        self.vec.get_spot(newdescr.pos, guard).compare_and_set(packing, NotValue, SeqCst, guard);
        
        return self.child.load(SeqCst, guard) != STATE_FAILED;
    }

    pub fn complete_pop_sub(&self, spot: Spot, old: Shared<usize>, descr: &PopSubDescr, guard: &Guard) -> bool {
        let cloned = descr.clone();
    
        let owned_descr = Owned::new(cloned).into_shared(guard);
    
        let cas_result = descr.parent.child
            .compare_and_set(Shared::<PopSubDescr>::null(), owned_descr, SeqCst, guard);
    
        if let Err(e) = cas_result {
            println!("The inserting the pop sub desc failed {:?}", e);
        }
    
        let result = if descr.parent.child.load(SeqCst, guard) == owned_descr {
            spot.compare_and_set(old, Owned::new(0).with_tag(TagNotValue), SeqCst, guard)
        } else {
            spot.compare_and_set(old, Owned::new(descr.value), SeqCst, guard)
        };
    
        if let Err(e) = result {
            println!("Something went wrong {:?}", e)
        }
    
        descr.parent.child.load(SeqCst, guard) == owned_descr
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
    pos: usize,
    child: Atomic<PopSubDescr>,
}

impl PopDescr {
    // vec: Atomic<WaitFreeVector>, 
    pub fn new(pos: usize) -> PopDescr {
        PopDescr {
            // vec,
            pos,
            child: Atomic::null(),
        }
    }
}

// PopSubDescr consists of a reference to a previously placed PopDescr (parent)
// and the value that was replaced by the PopSubDescr (value).
#[derive(Clone)]
pub struct PopSubDescr {
    parent: Rc<PopDescr>,
    value: usize,
}

// struct PopDescr {
//     vec: Rc<Vector>,
//     pos: usize,
//     state: u8,
// }