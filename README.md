# CraBar
A crabweight crab-scented bar based in smithay (WIP)

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

May be managed by issues instead of sections here.

[ ] Drop-in
    [ ] Options
        [x] `-h`: Thanks to clap it's out of box
        [ ] `-g`: 'Thanks' to wayland this may won't implement.
        [ ] `-b`: Low-priority: I just wondering who'll use it
        [ ] `-f`: Currently only Ubunto Mono
        [ ] `-p`: Low-priority: workaround: avoid broken pipe
        [ ] `-n`: Pending: maybe app-id?
        [ ] `-u`: Dependencies not satisfied: there're even no underlines
        [ ] `-B`: Another form: properly `#[args]`
        [ ] `-F`: Same as `-B`
        [ ] `-U`: Same as `-u`
    [ ] Formatting blocks
        [ ] `R`: Pending
        [ ] `l`: Pending (hard)
        [ ] `c`: Pending (hard)
        [ ] `r`: Pending (hard)
        [ ] `O`: Pending
        [x] `B`: Drop-in implemented
        [x] `F`: Drop-in implemented
        [ ] `T`: Dependencies not satisfied: no fonts for choosing
        [ ] `U`: Dependencies not satisfied: no underlines
        [x] `A`: Drop-in implemented
        [ ] `S`: Low-priority: same as `-b`
        [ ] Attributes
            [ ] `o`: Pending
            [ ] `u`: Pending
[ ] Lightweight
    [ ] Compile profile: Working
    [ ] Small dependencies: Pending
[ ] Speed
    [ ] Make it faster to recv/send data like 20KB/s (current 1KB/s)
