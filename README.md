# Chip 8 emulator

Chip8 emulator written in Rust, implements all standard opcodes following [this](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) reference.

### Running
Clone this repository and run
```
cargo run /path/to/rom
```
You can find some roms to test with [here](https://github.com/kripod/chip8-roms)
Note that there is no need to install SDL2 beforehand as the library will build it with the "bundled" flag.

### Resources
- https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

