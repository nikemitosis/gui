// unused for now
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Key {
    A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,
    N1,N2,N3,N4,N5,N6,N7,N8,N9,N0,
    Exclamation,At,Pound,Percent,Carat,Ampersand,Asterisk,BeginParen,EndParen,Hyphen,Underscore,Plus,Equals,
    Esc, Grave, Tilde, Tab, CapsLock, LShift, LCtrl, Fn, SysKey, LAlt,
    BeginSqrBracket,EndSqrBracket, BeginCurlyBracket, EndCurlyBracket, Pipe, Backslash, 
    Semicolon, Colon, Apostrophe, Quote, Enter, 
    Comma, Period, LessThan, GreaterThan, Slash, Question, RShift,
    RAlt, RCtrl, Menu,
    UpArrow,LeftArrow,DownArrow,RightArrow,
    Delete, Insert, Home, End, PageUp, PageDown, PrintScreen,
    F1,F2,F3,F4,F5,F6,F7,F8,F9,F10,F11,F12,
    NumLock, Np1, Np2, Np3, Np4, Np5, Np6, Np7, Np8, Np9, Np0, 
    NpUp,NpLeft,NpDown,NpRight, NpHome, NpEnd, NpPageUp, NpPageDown, NpInsert, NpEnter, 
    NpPlus,NpMinus,NpMul,NpDiv,NpDecimal,
}