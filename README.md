[![Rust](https://github.com/N3PDF/pineappl/workflows/Rust/badge.svg)](https://github.com/N3PDF/pineappl/actions?query=workflow%3ARust)
[![codecov](https://codecov.io/gh/N3PDF/pineappl/branch/master/graph/badge.svg)](https://codecov.io/gh/N3PDF/pineappl)
[![Documentation](https://docs.rs/pineappl/badge.svg)](https://docs.rs/pineappl)
[![crates.io](https://img.shields.io/crates/v/pineappl.svg)](https://crates.io/crates/pineappl)
[![DOI](https://zenodo.org/badge/248306479.svg)](https://zenodo.org/badge/latestdoi/248306479)

# Introduction

This repository contains libraries, tools, and interfaces to read and write
`PineAPPL` grids.

There are three crates in this repository:

- [`pineappl`](https://crates.io/crates/pineappl) is the crate containing the
  main functionality
- [`pineappl_capi`](https://crates.io/crates/pineappl) installs a library and a
  C header, to use PineAPPL inside a C program.
- [`pineappl_cli`](https://crates.io/crates/pineappl) installs a program to use
  PineAPPL from the command line.

# Installation

`PineAPPL` depends on [`Rust`](https://www.rust-lang.org/). If it's not already
installed on your system, use your favourite package manager to install it, or
go to <https://www.rust-lang.org/tools/install> and follow the instructions there.
