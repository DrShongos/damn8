# damn8
A CHIP-8 emulator written in Rust

To run, pass a path to a rom in the command line.

TODO:
- [ ] Use function pointers to handle opcodes instead of a large match statement.
- [ ] Support multiple CHIP-8 implementations.
- [ ] Add sound
- [ ] Configurable keypad
- [ ] Configurable colors
- [ ] Improve input handling
- [ ] Fix an issue with `index out of bounds` while rendering (might be related to the way adding is handled)