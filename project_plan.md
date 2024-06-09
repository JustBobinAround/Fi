# Project Plan

### What is FI?

FI is a play on the keystroke of VI to open vi, vim, or neovim.  It was
originally going to be a programming language that was going to stand for
"functionally imperative" as a funny oxymoron. However, as time has gone on, I
have realised there are enough joking and serious languages out there already.
I would come to decide that I should put my efforts towards more niche
topics and focus back on the roots of my interest in programming. FI stands
for "F--- it" I want to build a terminal emulator! And well, that's how this
all started.

I have always been fascinated by how terminal emulators and terminal
applications have worked since I was a kid. The origin of my fascination was
wanting to remake a terminal version of my favorite game that I liked to play,
Fate. Fate was a Diablo clone made by wild tangent games. A lot of the game's
configuration was stored in raw text and made it easily mod-able. I didn't know
what Diablo was at the time or that games like rogue existed. I would later
come to learn these facts and after struggling with many indexing errors
attempting to programm in c++ for the first time after only programming in
batch and ms-dos using only goto, I decided perhaps this was too hard. I was
around twelve or thirteen at the time. I would later move on to learn Java as
my primary language of choice, which sadly I would later come to find out was
not very suitable for complex terminal applications out-of-the-box. Anyways,
skip forward to college, I was bored out of my mind working on classes for crap
I already knew. Java, Python, stupid intro to web dev, etc. With all this free
time, I decided to learn Rust. Within the first two months, I had made an
interpreter for the lols, and then a fuzzy-finder terminal application. I was
sold on the language just due to the lower level access. I'm sure the same
thing would have happened if I had picked C++ or Go; this was just what hit
first. I guess I'm a part of the cult now.

### Overview of what the program should do

I want a terminal emulator that has a built in text editor. I want "normal mode"
to feel seamless between when being in the terminal and the editor so that
highlighting feels intuitive. I want the editor to be basically a copy of vim,
configured exactly how I like it. I haven't really changed my vim config for
like the past 3 years, so if I want to add a feature I should be able to just
add via source patch or just a toml file. Maybe if I feel up to it, I'll add
a scripting language. Maybe Python instead of lua to make the Brazilians mad lol.
If I do go the scripting route, I want to localize all plugins to be a part of
the actual source that are "opt-in" so I don't have to deal with all the crazy
plugin shenanigans that I do with neovim.

# Overview of Development 

### Phase 1 - Research

I have been creating a set programs that explore certain topics that I think will
pertain to the development of this project. I think the hope is that I explore
different part independently, and then combine and standardize the ideas into 
a draft product.

- Pty spawning and I/O
-   Ascii Escape Parsing
-   multiplexer
- higher level terminal UI api
- minimal terminal emulator
- minimal editor
- combined interfaces
- add configuration options
- reimplement in graphics accelerated version?

### Phase 2 - Spawning a Pty and Managing I/O

I have yet to figure out how to spawn a pty and deal I/O with only my own code.
Stuff never seems to work the way I think it will. For now I have decided to use
an external library, probably portable pty, as I think some of it might require
C bindings?. I don't really feel like dealing with this part for now. I might 
implement my own later.

### Phase 3 - Ascii Parsing

Once an initial implementation of the pty forker has been finished and has basic I/O
functions, Ascii parsing will need to be implemented. I will be testing the
functionality by separating the parser into its own process for now so that I
can pipe text with escape via bash using echo.

### Phase 3 - multiplexer

Once the parser has been made, implementing a multiplexer should be fairly easy.
I don't think much needs to be said here.

### Phase 4 - higher level terminal api

I really would like to implement something like tui-rs. I will probably be using
crossterm as a backend so that I don't have to think too hard about the escape
sequences.

### Phase 5 - Terminal emulator

Really this should basically be done in phase 4. I'd say the main feature here
is that I want to implement a global escape. In vi, you use the actual escape
key, which is fine, for the text editor part, I will leave it that way; That will
be the local escape key. For the global escape, the key will be ctrl+] as 
`ctrl+[` == escape on the keyboard. This key binding doesn't really seem to do anything
important. It should Also be noted that I remapped ctrl+space to be the escape
key because that allows me to quickly escape using both my thumbs on a kinesis
keyboard. I think for the global escape, I will remap ctrl+enter to send `ctrl+]`
and then also send ctrl+enter because I use that for slack a lot at my job.

