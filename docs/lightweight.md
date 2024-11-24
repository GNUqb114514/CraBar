# Lightweight
Lemonbar says that it's a *lightweight* bar,
and in my computer (x86_64, Arch Linux), it costs 27 kilobytes.
It's a small number, and it's hard to archieve,
but I'm still trying to make this crabby bar as small as possible.

## What did Lemonbar do
Lemonbar is a bar written in C with only 1609 lines,
that makes it very small. And it is running on an X server,
which doing almost everything about rendering for the programs.

## Why is CraBar so big
CraBar is written in Rust, a language that prefer statically
linking than dynamic linking, and take all things in a single binary
file.

## Current
The newest version costs 1.8 megabytes for featherweight & lightweight build,
and 3.5 megabytes for release build.

For comparing, Xwayland-run costs 2.1 megabytes,
`cage-satellite` costs 3.5 megabytes.
Considering they also costs a lot of time to figure it out that
they never actually docking lemonbar EVEN YOU SAY `-d`,
I think Crabar beats in Wayland.

For another comparing, a photo costs about 5 megabytes :)

## How to make it smaller?
First, *use featherweight & lightweight build*.
As I mentioned before, they can reduce 1.7 megabytes for you.

Second, *disable logs*.
Logs are helpful to debugging, but not helpful to perfomance.
Use `--no-default-features` to disable logs.
