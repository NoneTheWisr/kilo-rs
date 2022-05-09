# kilo-rs
This is my attempt at using the `rustea` crate.

I think I'm going to stop here, mid way through porting the code. I've ran into
an issue, which I don't believe is currently addressable.

Basically, `rustea` expects you to return a `String` from the `view` method
and provides no easy way to interact with the underlying `crossterm` backend.
That's fine for text styling, because you can just add the VT escape sequences
to that string, but it doesn't work so well for cursor movement, because the
cursor is stateful and `rustea`'s internal code causes changes to that state
outside of my control.

The way it's currently set up, I don't think there's much I can do. In order to
address that, I plan to make a fork and expose a `&mut impl Write` to the `view`
method instead.
