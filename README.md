![Discord](https://img.shields.io/discord/1381424110831145070?style=flat-square&logo=rust&link=https%3A%2F%2Fdiscord.gg%2FfHmRmuBDxF)

<p align="center">
  <a href="https://corte.si/posts/code/hilbert/portrait/">
    <img src="./assets/hilbert.png" alt="Hilbert curve illustration" />
  </a>
</p>
<p align="center">
  generated with: '<code>scurve allrgb hilbert</code>'
</p>


A **space-filling curve** is a continuous surjection $f:[0,1]\to[0,1]^d$ for
$d\ge 2$. In discrete spaces, this is an ordering of grid cells that visits
every cell; some orderings preserve adjacency (e.g., Hilbert), while others
trade adjacency for simplicity (e.g., Morton/Z-order).

This project contains implementations of various space-filling curves, plus
tools for visualising and working with them.


# spacecurve

[![crates.io](https://img.shields.io/crates/v/spacecurve.svg)](https://crates.io/crates/spacecurve)
[![docs.rs](https://docs.rs/spacecurve/badge.svg)](https://docs.rs/spacecurve)
[![MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A Rust library for generating and working with space-filling curves.


# scurve

[![crates.io](https://img.shields.io/crates/v/scurve.svg)](https://crates.io/crates/scurve)
[![MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)


A command-line tool and GUI for generating images of space-filling curves. 


# playground

A GUI for visualising space-filling curves and their properties, written in
Rust with [egui](https://github.com/emilk/egui) and compiled to WebAssembly.

<p align="center">
  <a href="https://corte.si/spacecurve/index.html">
    <img src="./assets/3d.png" alt="Space-filling curve viewer" />
  </a>
</p>



# related blog posts

Development on spacecurve (and its ancestors) is usually spurred along by posts
on my blog. Some of spacecurve's features are documented and illustrated in the
following posts:

- [Portrait of the Hilbert Curve](http://corte.si/posts/code/hilbert/portrait/index.html) 
- [Generating colour maps with space-filling curves](http://corte.si/posts/code/hilbert/swatches/index.html)
- [Hilbert Curve + Sorting Algorithms + Procrastination = ?](http://corte.si/posts/code/sortvis-fruitsalad/index.html)

# community

Want to contribute? Have ideas or feature requests? Come tell me about it on
[Discord](https://discord.gg/fHmRmuBDxF). 
