use crate::{Pixel,PixelIdx,window::SurfaceSlice};

pub trait GuiCell {
    fn draw(&self, surface: &mut SurfaceSlice);
}

pub struct SolidCell {
    color: Pixel,
} impl SolidCell {
    pub fn new(color: Pixel) -> Self {
        Self {color}
    }
} impl GuiCell for SolidCell {
    fn draw(&self, surface: &mut SurfaceSlice) {
        let size = surface.size();
        for y in 0..size.height() {
            for x in 0..size.width() {
                surface.set_pixel(PixelIdx{x,y},self.color);
            }
        }
    }
}


// gui cell types
// layer (front-back)
// split (vertical or horizontal)
// free-draw 
// swap (can be one of two or more cells depending on conditions)