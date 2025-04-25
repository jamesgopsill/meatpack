# Meatpack

A pure Rust implementation of Scott Mudge's [MeatPack][1] algorithm.
The crate works in both `std` and `no_std` environments. Add the `alloc` feature for additional for environments with a heap.
A CLI is provided and bindings for other languages are in the pipeline.
The `Packer` and `Unpacker` structs are configurable allowing you to set them up according to your embedded system resource constraints.

# Support

Please consider supporting the crate by:

- Downloading and using the crate.
- Raising issues and improvements on the GitHub repo.
- Recommending the crate to others.
- ⭐ the crate on GitHub.
- Sponsoring the [maintainer][2].

# The Algorithm

The gcode specification uses a restricted alphabet if ASCII (`u8`) characters resulting in files mostly made up of numbers, decimal point, a few letters ('G', 'M', 'E', etc.), and a few utility characters (newline, space, etc.).
Scott's histographic analysis of several gcode files found that **15 characters** forms **~93%** of all gcode.

**MeatPack** takes advantage of this histographic property and uses a 4-bit lookup table that matches the 15 most popular characters.
The 4-bit representations are then packed into 8-bit/1-byte characters effectively doubling the data density.

The 16th 4-bit character (`0b1111`) is reserved to tell the unpacker that a standard ASCII full-width character should be expected in the following byte. This allows the rest of the ASCII character set to be represented.

The algorithm features an additional optimisation where the packer reorders the characters if the full width character is surrounded by packable characters.
This helps avoid 4 bits being wasted when telling the packer that a full width character is coming up.

If the lower 4-bits contains `0b1111` then the full width character is unpacked followed by the upper 4-bit character.
If the upper 4-bit contains `0b1111` then the lower 4-bit character is unpacked followed by the full width character.
The reordering allows slightly more data to the be packed at the cost of a little complexity.

## Example

Take the following "G1" command.

`G1 X113.214 Y91.45 E1.3154`

which consists of 27 bytes (paranthesis indicate 1 byte),

`(G) (1) ( ) (X) (1) (1) (3) (.) (2) (1) (4) ( ) (Y) (9) (1) (.) (4) (5) ( ) (E) (1) (.) (3) (1) (5) (4) (\n)`

Applying the packing algorithm we can pack the command into 16 bytes as follows:

`(1G) (X ) (11) (.33) (12) ( 4) (9#) (Y) (.1) (54) (# ) (E) (.1) (13) (45) (\n)`

The characters are ordered in bit order (upper, lower) and would be unpacked lower then upper. With "Whitespace Removal" active, we can further reduce this to 13 bytes.

`(1G) (1X) (31) (2.) (41) (9#) (Y) (.1) (54) (1E) (3.) (51) (\n4)`

The packed command is now less than **half** the size of the original command.


# Command Patterns

MeatPack also provides a communication/control layer identified by 2 255 (`OxFF`) signal bytes followed by a command byte.
`0xFF` is virtually never found in gcode, so it is can be considered a reserved character.
The following command bytes exist:

| Byte (`u8`) Value | Command |
|---|---|
| 246 | Disable No Spaces |
| 247 | Enable No Spaces |
| 248 | Query Config |
| 249 | Reset All |
| 250 | Disable Packing |
| 251 | Enable Packing |
| 255 | Signal Byte |

# Examples

Examples can be found in the `examples` folder. No `alloc` featured examples can be called using:

```bash
cargo run --example pack
```

and `alloc` features.

```bash
cargo run --example alloc_unpack --features="alloc"
```


# References

- https://github.com/scottmudge/OctoPrint-MeatPack
- https://github.com/sponsors/jamesgopsill
- https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp
- https://github.com/scottmudge/Prusa-Firmware-MeatPack/blob/MK3_sm_MeatPack/Firmware/meatpack.cpp

[1]: https://github.com/scottmudge/OctoPrint-MeatPack
[2]: https://github.com/sponsors/jamesgopsill
[3]: https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp
[4]:  https://github.com/scottmudge/Prusa-Firmware-MeatPack/blob/MK3_sm_MeatPack/Firmware/meatpack.cpp
