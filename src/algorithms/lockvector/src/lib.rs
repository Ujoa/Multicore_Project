use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::ops::{Add, AddAssign};
#[derive(Debug)]
pub struct LockVector<T: Copy + Eq + Add + AddAssign> {
    pub list: Mutex<Vec<T>>,
    size: AtomicUsize,
}

impl <T> LockVector<T> 
where 
    T: Copy + Eq + Add + AddAssign
{
    pub fn new(size: usize) -> Self {
        LockVector {
            list: Mutex::new(Vec::with_capacity(size)),
            size: AtomicUsize::new(size),
        }
    }
    pub fn at(&self, index: usize) -> Option<T>{
        let val = match self.list.lock() {
            Ok(list) => {
                if index < list.len(){
                    return Some(list[index]);
                }
                None
            },
            Err(_) => {None},
        };
        val
    }

    pub fn push_back(&self, value: T){
        let list = &mut self.list.lock().unwrap();
        list.push(value);
    }
    
    pub fn pop_back(&self) -> Option<T> {
        let list = &mut self.list.lock().unwrap();
        list.pop()
    }

    pub fn erase(&self, index: usize) -> Option<T> {
        let list = &mut self.list.lock().unwrap();
        if index < list.len() {
            return Some(list.remove(index))
        }
        None
    }

    pub fn insertat(&self, index: usize, value: T){
        let list = &mut self.list.lock().unwrap();
        if index <= list.len() {
            list.insert(index, value);
        }
    }

    pub fn cwrite(&self, index: usize, old_value: T, new_value: T) -> bool{
        let list = &mut self.list.lock().unwrap();
        if index < list.len() {
            if old_value != new_value {
                list[index] = new_value;
                return true;
            }
        }
        false
    }

    pub fn length(&self) -> usize {
        let list = &mut self.list.lock().unwrap();
        list.len()
    }

    pub fn addat(&self, index: usize, val: T) {
        let list = &mut self.list.lock().unwrap();
        if index < list.len(){
            list[index] += val;
        }

    }


}
