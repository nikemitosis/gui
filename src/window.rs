mod windows;

#[cfg(feature="windows")]
use windows as win_sys;

pub use win_sys::init;

use win_sys::{InternalSurfaceData,WinHandle};

use std::marker::PhantomPinned;
use std::mem::MaybeUninit;
use std::pin::Pin;

use crate::{
    guicell::GuiCell,
    {CommonEvent,Direction,Distance,Partition,Partitionable,Pixel,PixelIdx,PixelIndexSlice,Size},
};

// assert internals are implemented correctly
mod _assertions {
    use super::*;
    
    const fn check_impls<
        WinHandle: InternalWindowTrait,
        Surface: InternalSurfaceTrait,
    >() {}
    
    const _: () = check_impls::<
        WinHandle,  // InternalWindowTrait
        Surface,    // InternalSurfaceTrait
    >();
    
    const _: unsafe fn(WinHandle) -> () = win_sys::close_window;
}

// what must be defined on WinHandle for Window to work properly
// it should be implemented in a separate, feature-blocked module based on the system
// closing logic should appear in drop, hence the requirement (and lack of close function in this trait)
// there should *always* be closing logic because the implementation of this trait fundamentally implies usage
//  of window system resources
#[allow(drop_bounds)]
trait InternalWindowTrait: Sized + Drop {
    fn new(name: &str, size: Size, owner: *mut Window) -> Result<Self,()>;
    
    // size of client area of window
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn size(&self) -> Size;
    
    // poll system for events
    // do not do event handling here or call handle_event directly
    fn check_events(&self);
}

type EventHandler = &'static dyn Fn(&mut dyn GuiCell, &CommonEvent) -> bool;
// ^return false when the default event handler should be skipped
pub struct Window {
    handle: Option<WinHandle>,
    event_handler: EventHandler,
    root: Box<dyn GuiCell>,
    surface: Surface,
    _pin: PhantomPinned,
} impl Window {
    // returns result on whether the window was successfully created
    // Windows must be pinned for window system
    pub fn new(
        name: &str, 
        size: Size, 
        root: Box<dyn GuiCell>, 
        event_handler: EventHandler,
    ) -> Result<Pin<Box<Self>>,()> {
        // DO NOT MOVE SELF
        let mut boxed: Box<MaybeUninit<Self>> = Box::<Self>::new_uninit();
        
        let self_ptr: *mut Self = (*boxed).as_mut_ptr();
        // ^safe so long as we don't read uninitialized
        
        let handle = WinHandle::new(name, size, self_ptr)?;
        
        unsafe {
            (&raw mut (*self_ptr).handle       ).write(Some(handle));
            (&raw mut (*self_ptr).event_handler).write(event_handler);
            (&raw mut (*self_ptr).root         ).write(root);
            (&raw mut (*self_ptr).surface      ).write(Surface::new((*self_ptr).handle.as_ref()));
            (&raw mut (*self_ptr)._pin         ).write(PhantomPinned);
        }
        
        Ok(Box::into_pin(unsafe {boxed.assume_init()}))
    }
    
    pub fn handle(&self) -> Option<&WinHandle> { self.handle.as_ref() }
    
    pub fn is_closed(&self) -> bool { self.handle.is_none() }
    
    // free all resources relating to this window and mark it as invalid
    pub fn close(&mut self) {
        if self.is_closed() { return; }
        
        let mut handle: Option<WinHandle> = None;
        
        std::mem::swap(
            &mut self.handle,
            &mut handle,
        );
        
        // handle.drop closes window
    }
    
    pub fn handle_events(&self) {
        if !self.is_closed() {
            self.handle().unwrap().check_events();
        }
    }
    
    pub fn draw(&mut self) {
        if self.is_closed() { return; }
        
        self.root.draw(self.surface.slice_mut());
        self.surface.commit(self.handle.as_ref().unwrap());
    }
    
    pub fn handle_common_event(&mut self, e: CommonEvent, internal: crate::private::Internal) {
        self.handle_event(e, internal);
    }
    
    // this signature will later change to accomodate custom events
    pub fn handle_event(&mut self, e: CommonEvent, _: crate::private::Internal) {
        if self.is_closed() { return }
        
        self.mandatory_event_prefix(&e);
        if (self.event_handler)(&mut *self.root, &e) {
            self.default_event_handler(&e);
        }
        self.mandatory_event_postfix(&e);
    }
    
    fn mandatory_event_prefix(&mut self, e: &CommonEvent) {
        match e {
            CommonEvent::Resize => { let _ = self.surface.update_size(self.handle.as_ref()); },
            _ => (),
        }
    }
    
    fn mandatory_event_postfix(&mut self, e: &CommonEvent) {
        match e {
            _ => ()
        }
    }
    
    // this function's signature *may* change in the future to accomodate custom events
    fn default_event_handler(&mut self, e: &CommonEvent) {
        match e {
            CommonEvent::Close => self.close(),
            CommonEvent::Draw  => self.draw(),
            
            _ => ()
        }
    }
} impl Drop for Window {
    fn drop(&mut self) {
        self.close();
    }
}

// must be implemented for surface to work
trait InternalSurfaceTrait {
    fn new(handle: Option<&WinHandle>) -> Self;
    
    fn commit(&self, handle: &WinHandle);
    
    fn deallocate(&mut self);
    fn reallocate(&mut self, size: Size) -> Result<(),()>;
}

struct Surface {
    internal: InternalSurfaceData,
    as_slice: SurfaceSlice,
    root: *mut Pixel,
    size: Size,
} impl Surface {
    pub fn slice(&self) -> &SurfaceSlice { &self.as_slice }
    pub fn slice_mut(&mut self) -> &mut SurfaceSlice { &mut self.as_slice }
    
    pub fn size(&self) -> Size { self.size }
    pub fn width (&self) -> usize { self.size().width  }
    pub fn height(&self) -> usize { self.size().height }
    
    pub fn root(&self) -> *mut Pixel { self.root }
    
    pub fn update_size(&mut self, handle: Option<&WinHandle>) -> Result<(),()> {
        let newsize = match handle {
            None => Size::ZERO,
            Some(h) => h.size(),
        };
        
        let size = self.size();
        if size != newsize {
            println!("Reallocating to size {:?}",size);
            self.reallocate(newsize)
        } else {
            Ok(())
        }
    }
    
} impl Drop for Surface {
    fn drop(&mut self) {
        self.deallocate();
    }
}

// invalid after Surface.resize
// never return an owned instance of a SurfaceSlice to prevent users from holding onto a 
//  dead slice. 
pub struct SurfaceSlice {
    domain: PixelIndexSlice,
    root: *mut Pixel,
    root_size: Size,
} impl SurfaceSlice {
    
    pub fn new(root: *mut Pixel, size: Size) -> Self {
        Self {
            domain: PixelIndexSlice {offset: PixelIdx::new(0,0), size: size},
            root: root,
            root_size: size,
        }
    }
    
    // idx *must* be in-bounds
    pub unsafe fn get_pixel_unchecked(&self, idx: PixelIdx) -> *mut Pixel {
        unsafe {
            self.root.add(idx.y * self.root_size.width() + idx.x)
        }
    }
    pub unsafe fn set_pixel_unchecked(&mut self,idx: PixelIdx, pixel: Pixel) {
        unsafe {*self.get_pixel_unchecked(idx) = pixel;}
    }
    
    pub fn get_pixel(&self,mut idx: PixelIdx) -> Pixel {
        idx += self.domain.offset;
        if !self.domain.contains(idx) { return Pixel::default(); }
        // ^ asserts that below call is safe (idx within domain)
        unsafe {*self.get_pixel_unchecked(idx)}
    }
    
    pub fn set_pixel(&mut self,mut idx: PixelIdx, pixel: Pixel) {
        idx += self.domain.offset;
        if !self.domain.contains(idx) { return; }
        // ^ asserts that below call is safe (idx within domain)
        unsafe {self.set_pixel_unchecked(idx,pixel);}
    }
    
    pub fn size(&self) -> Size { self.domain.size() }
    
} impl Default for SurfaceSlice {
    fn default() -> Self {
        Self {
            domain: PixelIndexSlice::default(),
            root: std::ptr::null_mut(),
            root_size: Size::default(),
        }
    }
} impl From<&Surface> for SurfaceSlice {
    fn from(surface: &Surface) -> Self {
        let size = surface.size();
        Self {
            domain: PixelIndexSlice {offset: PixelIdx::new(0,0), size: size},
            root: surface.root(),
            root_size: size,
        }
    }
} impl Partitionable for SurfaceSlice {
    fn partition(&self, dir: Direction, dst: Distance) -> Partition<Self> {
        let domain_partition = self.domain.partition(dir,dst);
        Partition {
            children: (
                Self {
                    root: self.root,
                    root_size: self.root_size,
                    domain: domain_partition.children.0,
                },
                Self {
                    root: self.root,
                    root_size: self.root_size,
                    domain: domain_partition.children.1,
                }
            ),
            direction: dir,
            distance: dst,
        }
    }
}