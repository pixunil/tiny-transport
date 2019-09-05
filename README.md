# Gtfs-Simulator

[![Build Status][travis-image]][travis-url]
[![Code Coverage][codecov-image]][codecov-url]

An experiment using open [Gtfs-Data of VBB][vbb-data] to render a minimalistic simulation with WebGL.

## Dependencies

This project uses [Rust][install-rust] and the [`wasm-pack` utility][install-wasm-pack].
For your convenience, the Gtfs data files are stored in the repo with [Git Lfs][git-lfs].


```bash
git lfs install
curl https://sh.rustup.rs -sSf | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Running

To run the simulation, you need to:

1. Convert the Gtfs-dataset
2. Compile the wasm module
3. Start a server
4. Open the page

```bash
cargo run --release import import/data/vbb.bzip
wasm-pack build wasm --target web --out-dir www/wasm
python3 -m http.server 8917
xdg-open http://localhost:8917/wasm/www
```

[travis-image]: https://travis-ci.org/pixunil/gtfs-sim.svg?branch=master
[travis-url]: https://travis-ci.org/pixunil/gtfs-sim
[codecov-image]: https://codecov.io/gh/pixunil/gtfs-sim/branch/master/graph/badge.svg
[codecov-url]: https://codecov.io/gh/pixunil/gtfs-sim
[vbb-data]: https://www.vbb.de/unsere-themen/vbbdigital/api-entwicklerinfos/datensaetze
[install-rust]: https://www.rust-lang.org/tools/install
[install-wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
[git-lfs]: https://git-lfs.github.com/
