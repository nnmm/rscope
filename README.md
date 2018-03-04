# rscope
A software X/Y oscilloscope intended for oscilloscope music

## What is rscope?
It's a simple program that takes audio from [JACK](http://www.jackaudio.org/) and displays the stereo signal as lines in a plane.
It uses a lot of code from the excellent [woscope](https://github.com/m1el/woscope).
m1el has a written a great explanation of how lines are rendered [here](http://m1el.github.io/woscope-how/).

It's not quite finished; I need to figure out why it still looks a little different from woscope.

## Running
First, install and start JACK. (That may be easier said than done.)

To run, after you have [installed Rust](https://www.rustup.rs/), run

    cargo run --bin rscope
    
Connect your audio source to rscope in JACK. That could be a VLC with the JACK plugin, software such as pd, or a PulseAudio bridge that provides all sound played on the system to JACK.

## Screenshot
Here's a heart created in a fork of pd, purr-data:
![Screenshot](/screenshot.jpg)
