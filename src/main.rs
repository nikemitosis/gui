use gui::*;

use crate::window::{Window};
use crate::guicell::{GuiCell,SolidCell};
use crate::Size;

// just use the default handler
fn handler(_cell: &mut dyn GuiCell, e: &CommonEvent) -> bool {
    println!("Receiving Event {:?}",e);
    true
}

fn main() -> Result<(),()> {
    
    unsafe {let _ = init();}
    
    let win = Window::new(
        "window", 
        Size {width: 800, height: 600}, 
        Box::new(SolidCell::new(
            Pixel {r: 0xFF, g: 0x00, b: 0x00, a: 0x00},
        )),
        &handler
    )?;
    
    while !win.is_closed() {
        win.handle_events();
    }
    
    Ok(())
}