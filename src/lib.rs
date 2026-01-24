#![allow(dead_code)]
#![allow(unused_parens)]

pub mod window;
pub mod guicell;
pub mod keys;

pub use window::init;

use std::ops::{Add,AddAssign};
use std::sync::OnceLock;

mod private { pub struct Internal; }

static INITIALIZED: OnceLock<()> = OnceLock::new();

pub struct Partition<T: Partitionable> {
    pub children: (T,T),
    pub direction: Direction,
    pub distance: Distance,
}

pub trait Partitionable: Sized {
    // result is left,right or top,bottom
    fn partition(&self,dir: Direction,dst: Distance) -> Partition<Self>;
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Distance {
    Relative(f32),
    Pixels(usize),
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum CommonEvent {
    Close,
    Draw,
    KeyDown(keys::Key),
    KeyUp(keys::Key),
    Maximize,
    Minimize,
    MouseMove,
    Resize,
    QueryByCursor,
    SetCursor,
    Shutdown,
}

// Custom events must be Send because threads can send events to each other's windows
// They must be 'static because they must implement Any in order to up/downcasted during transport
// POSTPONED FEATURE
/* #[derive(Clone,Copy)]
pub enum Event<T: 'static + Send> {
    BuiltIn(CommonEvent),
    CustomEvent(T),
} impl<T: 'static + Send> Event<T> {
    pub fn upcast(self) -> Event<Box<dyn Any + Send>> {
        match self {
            Event::BuiltIn(b)     => Event::BuiltIn(b),
            Event::CustomEvent(c) => Event::CustomEvent( Box::new(c) ),
        }
    }
} impl<T: 'static + Send> Into<Event<T>> for CommonEvent {
    fn into(self) -> Event<T> { Event::BuiltIn(self) }
} */

#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct Pixel {
    pub a: u8, pub r: u8, pub g: u8, pub b: u8, 
} impl From<u32> for Pixel {
    fn from(a: u32) -> Self {
        let mut rt = Pixel::default();
        unsafe { std::ptr::copy_nonoverlapping((&raw const a) as *const u8,(&raw mut rt) as *mut u8,4); }
        rt
    }
} impl From<Pixel> for u32 {
    fn from(a: Pixel) -> Self {
        let mut rt = 0u32;
        unsafe { std::ptr::copy_nonoverlapping((&raw const a) as *const u8,(&raw mut rt) as *mut u8,4); }
        rt
    }
}

#[derive(Clone,Copy,Debug,Default)]
pub struct PixelIdx {
    pub x: usize,
    pub y: usize,
} impl PixelIdx {
    pub fn new(x: usize, y: usize) -> Self { Self {x,y} }
} impl AddAssign for PixelIdx {
    fn add_assign(&mut self, a: Self) {
        self.x += a.x;
        self.y += a.y;
    }
} impl Add for PixelIdx {
    type Output = Self;
    fn add(mut self, b: Self) -> Self {
        self += b;
        self
    }
}

#[derive(Copy,Clone,Debug,Default)]
pub struct PixelIndexSlice {
    pub offset: PixelIdx,
    pub size: Size,
} impl PixelIndexSlice {
    
    pub fn size(&self) -> Size { self.size }
    
    pub fn contains(&self, idx: PixelIdx) -> bool {
        idx.x >= self.offset.x && idx.y >= self.offset.y 
        && idx.x < self.offset.x + self.size.width && idx.y < self.offset.y + self.size.height
    }
} impl Partitionable for PixelIndexSlice {
    fn partition(&self, dir: Direction, dst: Distance) -> Partition<Self> {
        
        let px_len: usize = match dst {
            Distance::Pixels(px) => px,
            Distance::Relative(p) => {
                (
                    match dir {
                        Direction::Horizontal => self.size.width,
                        Direction::Vertical => self.size.height
                    } as f64 * p as f64
                ) as usize
            }
        };
        
        Partition {
            children: match dir {
                Direction::Horizontal => (
                    Self {
                        offset: self.offset, 
                        size: Size {width: px_len, height: self.size.height},
                    },
                    Self {
                        offset: PixelIdx {x: self.offset.x + px_len, y: self.offset.y},
                        size: Size {width: self.size.width - px_len, height: self.size.height}
                    }
                ),
                Direction::Vertical => (
                    Self {
                        offset: self.offset,
                        size: Size {width: self.size.width, height: px_len},
                    },
                    Self {
                        offset: PixelIdx {x: self.offset.x, y: self.offset.y + px_len},
                        size: Size {width: self.size.width, height: self.size.height - px_len}
                    }
                ),
            },
            direction: dir,
            distance: dst,
        }
    }
}

// The purpose of this struct is to remove ambiguity of (usize,usize) 
// as that can be interpreted as rows,cols, or width,height
#[derive(Copy,Clone,Debug,Default,PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
} impl Size {
    
    pub const ZERO: Self = Size {width: 0, height: 0};
    
    pub const fn width (&self) -> usize { self.width  }
    pub const fn height(&self) -> usize { self.height }
    pub const fn rows  (&self) -> usize { self.height }
    pub const fn cols  (&self) -> usize { self.width  }
    
    pub const fn set_width (&mut self, value: usize) { self.width  = value; }
    pub const fn set_height(&mut self, value: usize) { self.height = value; }
    pub const fn set_rows  (&mut self, value: usize) { self.height = value; }
    pub const fn set_cols  (&mut self, value: usize) { self.width  = value; }
}