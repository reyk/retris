rETRIS
======

[![Build Status](https://travis-ci.org/reyk/retris.svg?branch=master)](https://travis-ci.org/reyk/retris)

reyk's very simple Tetris clone, written in Rust.

Introduction
------------

Let's start with an important disclaimer: I'm not a game developer. I
have actually never written a game before - if you forget about my
attempts at making custom DOOM II or Duke Nukem 3D levels a long time
ago.  So don't expect too much from this.

I wrote this game as a little _one-day_ exercise for three reasons:

1. To write something in Rust that is not related to my work or networking.
2. Because Tetris is fun!
3. To show my kids that programming is fun!

To my own surprise, my two kids (8 and 5 years old at this point) love
this game.  I have to admit that they've probably never heard about
Tetris before.  But they got addicted to it, and now we're beating our
highscores in the family.  What surprised me the most is that they
actually enjoy such a simple text-based and imperfect implementation
of the game.

And they were particularly excited about the fact that I wrote it
myself.  I'm a professional programmer all their lives and they never
really got interested in what I do.  But now they suddenly got it: I
write computer programs, and this can even be fun. My older one now
wants to learn how to code. Mission accomplished.

> Geek parenting tip: write a game for your kids to show them that
> programming can be fun and that computers are more than the apps or
> games on your device.

Usage
-----

Just run `cargo run` and follow the instructions.  It is that easy.

TODO
----

My kids have found a few things that can be optimized:

- The pixels are sometimes wrong on top of the field or when rotating a block.
- They would really like to have a stored high score table.
- Game music, but this is a bit out of my scope.

Furthermore:

- It doesn't support the hold buffer and a few other official Tetris rules.

Screenshot
----------

![rETRIS](retris.jpg?raw=true "rETRIS")
