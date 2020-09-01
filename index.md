---
layout: default
---

# Welcome to PineAPPL

**PineAPPL** is a computer library that makes it possible to produce fast-interpolation grids for fitting parton distribution functions (PDFs) including corrections of strong and electroweak origin.

[![Rust](https://github.com/N3PDF/pineappl/workflows/Rust/badge.svg)](https://github.com/N3PDF/pineappl/actions?query=workflow%3ARust)
[![codecov](https://codecov.io/gh/N3PDF/pineappl/branch/master/graph/badge.svg)](https://codecov.io/gh/N3PDF/pineappl)
[![Documentation](https://docs.rs/pineappl/badge.svg)](https://docs.rs/pineappl)
[![crates.io](https://img.shields.io/crates/v/pineappl.svg)](https://crates.io/crates/pineappl)
[![DOI](https://zenodo.org/badge/248306479.svg)](https://zenodo.org/badge/latestdoi/248306479)
[![Documentation Status](https://readthedocs.org/projects/pineappl/badge/?version=latest)](https://pineappl.readthedocs.io/en/latest/?badge=latest)
[![Anaconda-Server Badge](https://anaconda.org/conda-forge/pineappl/badges/installer/conda.svg)](https://anaconda.org/conda-forge/pineappl)

## How to download and install PineAPPL?

`PineAPPL` depends on [`Rust`](https://www.rust-lang.org/). If it's already
installed make sure that you have a recent version, otherwise the following
steps might break during compilations. If it's not installed yet, use your
favourite package manager to install it, or go to
<https://www.rust-lang.org/tools/install> and follow the instructions there.

Proceed by installing `cargo-c`, which is required by `pineappl_capi`:

    cargo install cargo-c

Next, install `pineappl_capi`:

    cd pineappl_capi
    cargo cinstall --release --prefix=${prefix}
    cd ..

and finally the command-line program:

    cargo install --path pineappl_cli

Make sure that all the required environment variables are set. See the
`README.md` of `pineappl_capi` for further instructions.

For the python interface please refer to the dedicated documentation
in [pineappl.readthedocs.io](https://pineappl.readthedocs.io/).

## Links to all the supporting documentation

- [Rust API documentation](https://docs.rs/pineappl)
- [C API reference](https://docs.rs/pineappl_capi/0.2.0/pineappl_capi/)
- [Python API](https://pineappl.readthedocs.io/)

## How to cite PineAPPL?

If you use the package please cite the following [Zenodo](https://zenodo.org/) and [arXiv](https://arxiv.org/) references:
- [10.5281/zenodo.3890291](https://doi.org/10.5281/zenodo.3890291)
- [arXiv:2008.12789](https://arxiv.org/abs/2008.12789)

## Links to the grids in the paper

The PineAPPL grids generated for the paper are available in a [dedicated git lfs repository](https://github.com/N3PDF/pineapplgrids). This repository contains the following relevant grids:

### ATLAS high-mass Drell-Yan lepton pair measurement at 7 TeV

<img src="images/pineappl_ATLASZHIGHMASS49FB.svg" style="width: 50%;"/>

### CMS double-differential Drell-Yan lepton pair measurement at 7 TeV

<img src="images/pineappl_CMSDY2D11_bin3.svg" style="width: 50%;"/><img src="images/pineappl_CMSDY2D11_bin4.svg" style="width: 50%;"/>
<img src="images/pineappl_CMSDY2D11_bin5.svg" style="width: 50%;"/><img src="images/pineappl_CMSDY2D11_bin6.svg" style="width: 50%;"/>

### ATLAS differential top-quark pair measurement at 8 TeV

<img src="images/pineappl_ATLAS_TTB_DIFF_8TEV_LJ_TPT.svg" style="width: 50%;"/><img src="images/pineappl_ATLAS_TTB_DIFF_8TEV_LJ_TTM.svg" style="width: 50%;"/>

### CMS differential Z pt measurement at 13 TeV

<img src="images/pineappl_CMS_Z_13_TEV.svg" style="width: 50%;"/>
