# MZ-GUI
This GUI Library aims to make GUI development as intuitive as possible through modular widgets, which are called "GuiCells" in this library, or sometimes just "Cells". 

## DISCLAIMER
This is in very early development (it doesn't even work at the moment I'm writing this) so *everything* is subject to change, and most likely will (or already has and I haven't gotten around to rewriting the description here)


### TODO

#### Bugs
- Window doesn't draw
- Resizing breaks window

#### Technical
- Add multithreading support
  - Windows
    - Change `GWLP_USERDATA` to point to a lock
- Unix support

#### Features
- Add custom events
- Add more common widgets
- Icons
- Min/max window sizes
- Fullscreen + Borderless
- Recognize window focus

## Gui Cells
A gui cell is simply anything implementing the `GuiCell` trait. You can create your own `GuiCell` variants, or you can use ones provided by the library. The full definition of `GuiCell` is below: 

```rs
pub trait GuiCell {
    fn draw(&self, surface: &mut SurfaceSlice);
}
```

All gui cells are responsible for a rectangular slice of the window, represented as a `SurfaceSlice`. Often times, a gui cell may delegate subslices to child cells. More on `Surface`s below. 

## Window
The `Window` struct is the struct used when instantiating GUIs

Every window has a root gui cell, whose slice covers the entire window (not including the toolbar). You determine what type of cell you want the root cell to be statically via the generic argument in `Window`. 

Windows are identified by their ID, found by calling `<window>.id()`. Window IDs are needed in order to send a window custom events. 

Note that window ids are very simple and can be easily obtained by simply guessing a number between 0 and the current number of existing windows. 

### Surface
The `Surface` of a window is the 2-dimensional array of pixels that is displayed on the window. As you may have noticed, implementors of `GuiCell` do not work with surfaces directly. Instead, implementors work with `SurfaceSlice`s

## Events
### Common Events
`CommonEvent`s, also known as built-in or system events are events sent to the window by the window system. These can be requests, like to close the window (`CommonEvent::Close`) or simply informational, like `CommonEvent::Shutdown`, which signals that the window is being closed. 

### Custom Events
Custom events can be of any type, but must be `Send`. They are specified statically by the second generic parameter of `Window`. Windows can be sent events from anywhere using `send_event(<WinID>, <event>)`

Because `WinID`s are so predictable, do not assume arbitrary code won't be able to send custom events to your window if you're building a library. 

### Event Handlers
Each window has exactly one event handler, determined statically by the third generic parameter of `Window`. Event handlers must implement `Fn(&mut Window<...>, &Event) -> bool`, where the first parameter is a mutable reference to the window receiving the event, the `&Event` is a reference to the event being received, and the return value is a `bool` indicating whether or not the default handler should be invoked. 

Some system events always trigger some action internally, such as `CommonEvent::Resize`, which always resizes the window's surface when appropriate


