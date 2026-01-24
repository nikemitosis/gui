mod win32;

use crate::*;
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{HBITMAP,HDC}
};
use super::{
    InternalWindowTrait, Window, Surface, SurfaceSlice, InternalSurfaceTrait
};

pub use win32::init;

// WinHandle is soft-guaranteed to point to a valid window
//  This guarantee is broken when Into<HWND> is misused to invalidate the underlying window behind WinHandle's back
//  or any way HWND is obtained and misused. This can only happen through unsafe calls to the Win32 library
#[derive(Debug,Eq,PartialEq)]
pub struct WinHandle(HWND);
impl InternalWindowTrait for WinHandle {
    fn new(name: &str, size: Size, owner: *mut Window) -> Result<Self,()> {
        Ok(Self(win32::create_window(name,size.width as i32,size.height as i32,owner)?))
    }
    
    fn size(&self) -> Size {
        unsafe { win32::get_win_size(self.0) }
    }
    fn width (&self) -> usize { self.size().width  }
    fn height(&self) -> usize { self.size().height }
    
    fn check_events(&self) {
        unsafe {win32::check_messages(Some(self.0))}
    }
} impl Drop for WinHandle {
    fn drop(&mut self) {
        unsafe {win32::close_window(self.0);}
    } 
} impl From<&WinHandle> for HWND {
    fn from(a: &WinHandle) -> Self { a.0 }
} impl std::hash::Hash for WinHandle {
    fn hash<H: std::hash::Hasher>(&self,state: &mut H) {
        self.0.0.hash(state); // use the hash of the *mut c_void
    }
}


pub struct InternalSurfaceData {
    pub mem_hdc: HDC,
    pub dib_bitmap_handle: HBITMAP,
} impl InternalSurfaceData {
    pub fn new(handle: Option<&WinHandle>) -> Result<(Self, *mut Pixel, Size),()> {
        
        let size = match handle {
            None => panic!("Tried to create a surface with no handle"),
            Some(h) => h.size(),
        };
        
        let (hbitmap, hdc, pixels) = win32::allocate_dib(size)?;
        
        Ok((
            Self {
                mem_hdc: hdc,
                dib_bitmap_handle: hbitmap,
            },
            pixels,
            size,
        ))
    }
}

impl InternalSurfaceTrait for Surface {
    fn new(handle: Option<&WinHandle>) -> Self {
        
        let (internal, root, size) = InternalSurfaceData::new(handle).expect(
            "Could not allocate DIB for InternalSurface"
        );
        
        Self {
            internal,
            as_slice: SurfaceSlice::new(root,size),
            root,
            size,
        }
    }
    
    fn reallocate(&mut self, size: Size) -> Result<(),()> {
        let realloc_result = win32::allocate_dib(size);
        
        match realloc_result {
            Err(_) => {
                eprintln!("Unable to reallocate DIB. Try resizing the window? No promises."); 
                Err(())
            },
            Ok(tup) => {
                self.deallocate();
                (self.internal.dib_bitmap_handle, self.internal.mem_hdc, self.root) = tup;
                self.as_slice = SurfaceSlice::from(&*self);
                Ok(())
            }
        }
    }
    fn deallocate(&mut self) {
        if self.root == std::ptr::null_mut() { return; }
        unsafe { win32::release_dib(self.internal.dib_bitmap_handle, self.internal.mem_hdc) }
        self.root = std::ptr::null_mut();
        self.as_slice = SurfaceSlice::default();
    }
    
    fn commit(&self, handle: &WinHandle) {
        unsafe {
            win32::blit_dib(
                handle.into(),
                self.internal.dib_bitmap_handle, 
                self.size(), 
                self.internal.mem_hdc
            );
        }
    }
    
}


pub unsafe fn close_window(handle: WinHandle) {
    unsafe {win32::close_window(handle.0)}
}