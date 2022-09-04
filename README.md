Seven-segment clock (`7clock`)
==============================

https://user-images.githubusercontent.com/21787/185858001-8806a8fd-fe48-458e-ad59-2c03f0b3a339.mp4

This is a clock for terminals that uses the Unicode seven-segment display characters added in Unicode 13.0.
It runs on most commonly used operating systems, including BSD, Linux, macOS, and Windows.

You need to have a font installed that has glyphs for the seven-segment display characters. I use
[PragmataPro](https://fsd.it/shop/fonts/pragmatapro/). Another option is [Iosevka](https://typeof.net/Iosevka/).

**Note:** It doesn't actually cycle through colours when it's running, that was just for demonstration in the video.

Building
--------

Ensure you have [installed the Rust compiler][install-rust], then:

```
cargo build --release --locked
```

Running
-------

After building the binary will be at `target/release/7clock`.

Run the binary to get the default 12-hour clock without seconds. The following
options are supported:

* `-24` — use 24-hour time
* `--colour` — set the colour of the clock (see `--help` for more info)
* `--seconds` — display seconds

Credits
-------

* This clock is partially inspired by [clock-tui](https://github.com/race604/clock-tui)

Licence
-------

This project is dual licenced under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/wezm/7clock/blob/master/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/wezm/7clock/blob/master/LICENSE-MIT))

at your option.

[install-rust]: https://www.rust-lang.org/learn/get-started
