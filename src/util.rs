use self::Space::*;

use std::{
    ops::{Deref,DerefMut,Index,IndexMut},
    sync::RwLock,
};

fn mutify<T>(rf: &T) -> *mut T {
    rf as *const T as *mut T
}

enum Space<T> {
    Vacant(usize),
    Occupied(T),
} impl<T> Space<T> {    
    pub fn occupy(&mut self, occupant: T) -> usize {
        if self.is_occupied() {
            panic!("Tried to occupy an already-occupied space");
        }
        let mut swapper = Self::Occupied(occupant);
        std::mem::swap(self, &mut swapper);
        if let Self::Vacant(rt) = swapper {
            return rt;
        }
        panic!("Impossible state; the program should not be able to reach this");
    }
    pub fn vacate(&mut self, next_open: usize) -> T {
        if self.is_vacant() {
            panic!("Tried to vacate an already-vacant space");
        }
        let mut swapper = Self::Vacant(next_open);
        std::mem::swap(self, &mut swapper);
        if let Self::Occupied(dat) = swapper {
            return dat;
        }
        panic!("Impossible state; the program should not be able to reach this");
    }
    
    pub fn is_vacant  (&self) -> bool { if let Vacant  (_) = self { true } else { false } }
    pub fn is_occupied(&self) -> bool { !self.is_vacant() }
    
    pub fn get(&self) -> Option<&T> {
        match self {
            Vacant(_) => None,
            Occupied(dat) => Some(dat),
        }
    }
    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Vacant(_) => None,
            Occupied(dat) => Some(dat),
        }
    }
}

pub mod unsync {
    use super::*;
    
    pub struct RecycleList<T> {
        next_open: usize,
        dat: Vec<Space<T>>,
    } impl<T> RecycleList<T> {
        pub const fn new() -> Self {
            Self {
                next_open: 0,
                dat: Vec::new(),
            }
        }
        
        pub fn len(&self) -> usize { self.dat.len() }
        
        pub fn get(&self, idx: usize) -> Result<&T,()> {
            if let Some(d) = self.dat[idx].get() {
                Ok(d)
            } else {
                Err(())
            }
        }
        
        pub fn get_mut(&mut self, idx: usize) -> Result<&mut T,()> {
            if let Some(d) = self.dat[idx].get_mut() {
                Ok(d)
            } else {
                Err(())
            }
        }
        
        pub fn contains(&self, idx: usize) -> bool { self.dat[idx].is_occupied() }
        
        pub fn insert(&mut self, item: T) -> usize {
            let idx = self.next_open;
            if self.next_open == self.len() {
                self.dat.push(Space::Occupied(item));
                self.next_open += 1;
            } else {
                self.next_open = self.dat[idx].occupy(item);
            }
            return idx;
        }
        
        pub fn remove(&mut self, idx: usize) -> T {
            let rt = self.dat[idx].vacate(self.next_open);
            self.next_open = idx;
            rt
        }
        
    } impl<T> Index<usize> for RecycleList<T> {
        type Output = T;
        fn index(&self, idx: usize) -> &Self::Output {
            match self.get(idx) {
                Ok(d) => d,
                Err(_) => panic!("No element exists at index {}",idx),
            }
        }
    } impl<T> IndexMut<usize> for RecycleList<T> {
        fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
            match self.get_mut(idx) {
                Ok(d) => d,
                Err(_) => panic!("No element exists at index {}",idx),
            }
        }
    }
}

pub mod sync {
    use super::*;
    
    
    // Nested lock structure for syncing:
    // Each (occupied) element has its own RwLock
    // And the list itself has its own RwLock (for things like inserting/removing elements) 
    // Vacancies do not count as elements, they count as part of the list itself
    
    pub struct RecycleList<T> {
        next_open: usize, // no syncing needed here; only accessed when dat_lock.write() is accessed
        dat: Vec<Space<RwLock<T>>>,
        dat_lock: RwLock<()>, 
    } impl<T> RecycleList<T> {
        pub const fn new() -> Self {
            Self {
                next_open: 0,
                dat: Vec::new(),
                dat_lock: RwLock::new(()),
            }
        }
        
        fn dat(&self) -> *mut Vec<Space<RwLock<T>>> { mutify(&self.dat) }
        fn next_open(&self) -> *mut usize { mutify(&self.next_open) }
        
        pub fn len(&self) -> usize {
            let _a = self.dat_lock.read().unwrap();
            self.dat.len()
        }
        
        pub fn get(&self, idx: usize) -> Result<impl Deref<Target=T>,()> {
            let _a = self.dat_lock.read().unwrap();
            if let Some(d) = self.dat[idx].get() {
                Ok(d.read().unwrap())
            } else {
                Err(())
            }
        }
        
        pub fn get_mut(&self, idx: usize) -> Result<impl DerefMut<Target=T>,()> {
            let _a = self.dat_lock.read().unwrap();
            if let Some(d) = self.dat[idx].get() {
                Ok(d.write().unwrap())
            } else {
                Err(())
            }
        }
        
        pub fn contains(&self, idx: usize) -> bool {
            let _a = self.dat_lock.read().unwrap();
            self.dat[idx].is_occupied()
        }
        
        // interior mutability synced by dat_lock
        pub fn insert(&self, item: T) -> usize {
            let _a = self.dat_lock.write().unwrap();
            
            let idx: usize;
            unsafe {
                let dat = self.dat();
                let next_open = self.next_open();
                
                idx = *next_open;
                let new_item = RwLock::new(item);
                if *next_open == (*dat).len() {
                    (&mut *dat).push(Space::Occupied(new_item));
                    *next_open += 1;
                } else {
                    *next_open = (&mut *dat)[idx].occupy(new_item);
                }
            }
            
            return idx;
        }
        
        // interior mutability synced by dat_lock
        pub fn remove(&self, idx: usize) -> Option<T> {
            let _a = self.dat_lock.write().unwrap();
            
            let rt: T;
            unsafe {
                let dat = self.dat();
                let next_open = self.next_open();
                
                if (&mut *dat)[idx].is_vacant() { return None; }
                
                rt = (&mut *dat)[idx].vacate(*next_open).into_inner().unwrap();
                *next_open = idx;
            }
            Some(rt)
        }
        
    } unsafe impl<T> Sync for RecycleList<T> {}
}