# MZ-GUI
This GUI library aims to make GUI development as intuitive as possible through modular widgets, which are called "GuiCells" in this library, or sometimes just "Cells". 

It aims to be cross-platform in the future, but for now only supports Windows. 

## DISCLAIMER
This is in *very* early development so *everything* is subject to change, and most likely will (or already has and I haven't gotten around to rewriting the description here)

There is lots to do before this is becomes a library I can actually recommend using. 

Just to name a few things:
- Linux/Mac support
- Text/Font rendering
- Multithreading support
- Utilize graphics card
- Child windows (and inter-window communication)
- User-defined events
- More common widgets
- Window Icons
- clamp window sizes
- Fullscreen and/or borderless
- Recognize window focus

## Gui Cells
A gui cell is simply anything implementing the `GuiCell` trait. You can create your own `GuiCell` variants, or you can use ones provided by the library. The full definition of `GuiCell` is below: 

```rs
// Outdated!
pub trait GuiCell {
    fn draw(&self, surface: &mut SurfaceSlice);
}
```

All gui cells are responsible for a rectangular slice of the window, represented as a `SurfaceSlice`. Often times, a gui cell may delegate subslices to child cells. More on `Surface`s below. 

## Window
The `Window` struct is the struct used when instantiating GUIs and managing. 

Every window has a root gui cell, whose slice covers the entire window (not including the toolbar). You determine what type of cell you want the root cell to be statically via the generic argument in `Window`. 


### Surface
The `Surface` of a window is the 2-dimensional array of pixels that is displayed on the window. As you may have noticed, implementors of `GuiCell` do not work with surfaces directly. Instead, implementors work with `SurfaceSlice`s. If you want to implement your own 

## Events

### Common Events
`CommonEvent`s, also known as built-in or system events are events sent to the window by the window system. These can be requests, like to close the window (`CommonEvent::Close`), or simply informational, like `CommonEvent::Shutdown`, which signals that the window *is* being closed. 

### Custom Events
IN PROGRESS - Check back soon!

### Event Handlers
(Forewarning: This information may soon be outdated)

Each window has exactly one event handler. Event handlers must implement `Fn(&mut Window<...>, &Event) -> bool`, where the first parameter is a mutable reference to the window receiving the event, the `&Event` is a reference to the event being received, and the return value is a `bool` indicating whether or not the default handler should be invoked. 

Some system events always trigger some action internally, such as `CommonEvent::Resize`, which always resizes the window's surface when appropriate
