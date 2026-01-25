use std::ffi::c_void;

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
    }
};

use crate::*;
use crate::window::Window;

use std::result::Result as Result;

// wrappers around windows functions for windows.rs

pub const CLASS_NAME: PCWSTR = w!("mz-gui");

// events we'd like the library to handle - for when the system default is not adequete
#[derive(Copy,Clone,Debug)]
enum InternalEvent {
    WindowResize,
}
#[derive(Copy,Clone,Debug)]
enum WndProcEvent {
    Internal(InternalEvent),
    Common(CommonEvent),
    Unknown,
}

// Error returns whether event was recognized at all
// false means ignored, true means handled by library
const fn translate_message(msg: u32, wparam: usize, lparam: isize) -> WndProcEvent {
    use self::{
        WndProcEvent::*,
        InternalEvent::*,
    };
    use crate::CommonEvent::*;
    
    match msg {
        WM_DESTROY  => Common(Shutdown),
        WM_CLOSE    => Common(Close),
        WM_PAINT    => Common(Draw),
        WM_SIZE     => Common(Resize),
        
        /* WM_WINDOWPOSCHANGING => {
            let winpos: WINDOWPOS = unsafe {*(lparam as *const WINDOWPOS)};
            let flags: u32 = winpos.flags.0;
            let is_resize = (!flags & SWP_NOSIZE.0) != 0;
            if is_resize {
                Internal(WindowResize)
            } else {
                Unknown
            }
        }, */
        // incomplete
        
        // (
              // WM_MOUSEFIRST
            // | WM_SETCURSOR
            // | WM_NCHITTEST
            // | WM_NCLBUTTONDBLCLK
            // | WM_NCMOUSELEAVE
            // | WM_NCMOUSEMOVE
            // | WM_WINDOWPOSCHANGED
            // | WM_GETICON // user later
            // | WM_MOVE // use later
            // | WM_MOVING // or maybe this instead
            // | WM_SETTEXT
            // | WM_ACTIVATE // use later
            // | WM_SYSKEYDOWN
            // | WM_GETMINMAXINFO // use later
            // | WM_CAPTURECHANGED
            // | WM_MOUSEWHEEL // use later as a key input(?)
            // | WM_MOUSEHWHEEL // ^
        // ) => Err(EventReaction::UseDefault),
        _ => Unknown,
    }
}

pub fn get_hinstance() -> Result<HINSTANCE,()> {
    match unsafe {GetModuleHandleW(None)} {
        Ok(hinst) => Ok(hinst.into()),
        Err(_) => Err(()),
    }
}

fn load_cursor() -> Result<HCURSOR,()> {
    match unsafe {LoadCursorW(None,IDC_ARROW)} {
        Ok(cursor) => Ok(cursor),
        Err(_) => {
            eprintln!("Unable to load cursor");
            Err(())
        },
    }
}
fn init_win_class() -> Result<(),()> {
    let cursor = load_cursor()?;
    
    let win_class = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),    // long-pointer to function window proceudre
        hInstance: get_hinstance()?,    // claim responsibility/knowledge of this window(?)
        lpszClassName: CLASS_NAME,      // name of window class
        hCursor: cursor,                // cursor used by windows of this class
        hbrBackground: HBRUSH(std::ptr::null_mut() as *mut c_void), // don't make a background for us
        ..Default::default()
    };
    
    if ( unsafe {RegisterClassW(&win_class)} != 0) {
        Ok(())
    } else {
        eprintln!("Unable to register window class");
        Err(())
    }
}

// this function must ONLY be run by a single thread, before any windows are created
pub unsafe fn init() -> Result<(),()> {
    if INITIALIZED.set(()).is_err() {
        return Ok(());
    }
    init_win_class()?;
    
    Ok(())
}

pub fn create_window(name: &str, width: i32, height: i32, owner: *mut Window) -> Result<HWND,()> {
    if INITIALIZED.get().is_none() {
        panic!("Attempted to create a window before system was intialized. Call {}::init() before attempting to create any guis", env!("CARGO_PKG_NAME"))
    }
    
    let create_result = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            PCWSTR::from_raw(HSTRING::from(name).as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            Some(get_hinstance()?),
            None,
        )
    };
    
    let hwnd = match create_result {
        Ok(h) => h,
        Err(_) => return Err(()),
    };
    
    let _ = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, owner as isize) };
        
    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
    if !<BOOL as Into<bool>>::into(unsafe { UpdateWindow(hwnd) }) {
        eprintln!("Windows Syscall Error - UpdateWindow fail");
        return Err(())
    }
    
    Ok(hwnd)
}

pub unsafe fn check_messages(hwnd: Option<HWND>) {
    let mut msg = MSG::default();
    while <BOOL as Into::<bool>>::into(unsafe {PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE)}) {
        unsafe {
            let _ = TranslateMessage(&mut msg); // convert 'KEY_W' to 'w', for example
            DispatchMessageW(&msg);
        }
    }
}

pub unsafe fn close_window(hwnd: HWND) {
    unsafe {let _ = DestroyWindow(hwnd);}
}

pub unsafe fn release_dib(hbm: HBITMAP, hdc: HDC) {
    unsafe { 
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc);
    }
}

pub fn allocate_dib(size: Size) -> Result<(HBITMAP, HDC, *mut Pixel),()> {
    
    if size == Size::ZERO {
        panic!("Cannot create a zero-sized DIB");
    }
    
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFO>() as u32;
    bmi.bmiHeader.biWidth = size.width() as i32;
    bmi.bmiHeader.biHeight = -(size.height() as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;
    
    
    let hdc = unsafe {CreateCompatibleDC(None)};
    if hdc.0.is_null() {
        panic!("Couldn't create compatible device context to allocate DIB");
    }
    
    let mut pixels: *mut std::ffi::c_void = std::ptr::null_mut();
    let dib_bitmap_handle = unsafe {CreateDIBSection(
        Some(hdc),
        &raw const bmi,
        DIB_RGB_COLORS,
        &raw mut pixels,
        None,
        0
    )};
    
    if dib_bitmap_handle.is_err() {
        panic!("Could not allocate DIB");
    }
    
    Ok((dib_bitmap_handle.unwrap(), hdc, pixels as *mut Pixel))
}

pub unsafe fn get_window_rect(hwnd: HWND) -> PixelIndexSlice {
    let mut rect = RECT::default(); 
    unsafe { let _ = GetWindowRect(hwnd, &mut rect); }
    
    PixelIndexSlice {
        offset: PixelIdx {x: rect.left as usize, y: rect.top as usize},
        size: Size {
            width:  (rect.right - rect.left) as usize, 
            height: (rect.bottom - rect.top) as usize, 
        }
    }
}

pub unsafe fn get_client_rect(hwnd: HWND) -> PixelIndexSlice {
    let mut rect = RECT::default(); 
    unsafe { let _ = GetClientRect(hwnd, &mut rect); }
    
    PixelIndexSlice {
        offset: PixelIdx {x: rect.left as usize, y: rect.top as usize},
        size: Size {
            width:  (rect.right - rect.left) as usize, 
            height: (rect.bottom - rect.top) as usize, 
        }
    }
}

pub unsafe fn get_win_size(hwnd: HWND) -> Size {
    unsafe {get_client_rect(hwnd).size}
}

pub unsafe fn blit_dib(hwnd: HWND, hdib: HBITMAP, size: Size, src_dc: HDC) {
    let mut ps = PAINTSTRUCT::default();
    unsafe {
        let old = SelectObject(src_dc,hdib.into());
        let dst_dc = BeginPaint(hwnd, &mut ps);
        let _ = BitBlt(dst_dc, 0, 0, size.width as i32, size.height as i32, Some(src_dc), 0, 0, SRCCOPY);
        let _ = SelectObject(src_dc, old);
        let _ = EndPaint(hwnd, &ps);
    }
}

// safe if valid HWND is valid + its GWLP_USERDATA points to this library's window struct 
// i.e. this library owns the given HWND
pub unsafe fn get_window(hwnd: HWND) -> *mut Window {
    unsafe {
        GetWindowLongPtrW(hwnd,GWLP_USERDATA) as *mut Window
    }
}

extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    
    // need to change resize event handling
    // don't use WM_SIZE
    // GetWindowRect + GetClientRect upon WM_WINDOWPOSCHANGING?
    let translation = translate_message(msg,wparam.0,lparam.0);
    if let WndProcEvent::Unknown = translation {} else {
        println!("{:?}",translation);
    }
    match translation {
        WndProcEvent::Common(common) => unsafe {
            let window = get_window(hwnd);
            if window != std::ptr::null_mut() {
                (*window).handle_event(common, crate::private::Internal);
            }
            LRESULT(0)
        },
        WndProcEvent::Internal(internal) => match internal {
            _ => unsafe { DefWindowProcW(hwnd,msg,wparam,lparam) },
        }
        WndProcEvent::Unknown => {
            // println!("Unrecognized event with id {:02X}",msg);
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        },
    }
}