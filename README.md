# CraBar
A crabweight crab-scented bar based in smithay.

## This isn't
- A drop-in replacement for lemonbar (but it will be);
- A bar in X11 (it's just for Wayland)

## Goal
Current: Be a drop-in replacement for lemonbar in Wayland
Further: Be better than lemonbar

## About lightweight
(This may be moved into a separated file.)
Lemonbar says that it's a *lightweight* bar,
and in my computer (x86_64, Arch Linux), it costs 27 kilobytes.
It's a small number, and it's hard to archieve.
The newest version (release build) costs 3.4 megabytes,
and for comparing, Xwayland-run costs 2.1 megabytes,
`cage-satellite` costs 3.5 megabytes.
Considering they also costs a lot of time to figure it out that
they never actually docking lemonbar EVEN YOU SAY `-d`,
I think this solution is sorta 'lightweight'.

Further, I'm considering should I put a modified version in compile profiles to reduce compiling size.

## To-Do

Now managed by [issues](https://github.com/GNUqb114514/CraBar/issues) and [milestones](https://github.com/GNUqb114514/CraBar/milestones).
