# TRS-80 Casette Data Formats

## Data Representation

## File Formats

After decoding the binary data from the casette audio, there will be two or more blocks.
The first will always be a header block and the remaining will all be data blocks.
Each block should have 256 0x55 (0b01010101) bytes, then a sync byte of 0x7F (0b011111111).
This should be striped out prior to decoding, and will be omitted in the remainder of this document.
If you happen to use the [ImHex](https://imhex.werwolv.net) hex editor, you can use the provided hexpat pattern files in this directory ([data.hexpat](data.hexpat) and [header.hexpat](header.hexpat)).

![Hex editor view of header block](https://github.com/Basicprogrammer10/trs80-interface/assets/50306817/620a3604-00ef-4278-ac3a-406b8c6c3fc4)

The first byte (0x00) is the type of data contained in the block, below is a table of possible values.

| Byte | File | Description        |
| ---- | ---- | ------------------ |
| 0x9C | .DO  | Document           |
| 0xDO | .CO  | Compiled Software? |
| 0xD3 | .BA  | BASIC Program      |

The next 6 bytes are the file name, if the name is shorter than 6 bytes, the remaining bytes are padded with spaces.

After that there are 10 miscellaneous bytes, after all my research, I still have no idea what these are for.
Just ignore them I guess?

Then a checksum byte.
After adding all of the data bytes (0x01-0x10) with the checksum byte 0x11, the result mod 8 should be zero.
If not, something has gone wrong!
In rust we can do something like this to calculate the checksum.

```rust
data[0x01..=0x11].iter().fold(0_u8, |acc, &x| acc.wrapping_add(x))
```

Finally, at the end there should be 14 0x00 bytes.
This is true for every block, not just the header block.

### Text Data

![Hex editor of a text data block](https://github.com/Basicprogrammer10/trs80-interface/assets/50306817/3c025531-7403-47d3-b6fa-42a74d86daaa)

Now onto the data blocks!
There are a bit simpler.
Every data block should start with a 0x8D, then 256 bytes of data, a checksum byte, and 14 0x00 bytes.

## Further Reading

- https://github.com/lkesteloot/trs80/tree/master/packages/trs80-cassettes &ndash; Covers decoding the audio data including information for a low speed system used in some other TRS systems
- [Oppedahl, Carl. Inside the TRS-80 Model 100. Weber System, 1985.](http://www.club100.org/library/libdoc.html) &ndash; Good information on the whole casette system, but only briefly mentions the file formats
