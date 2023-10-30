**SpeedKetchup**
------------------------------

Automatically run periodic internet [speed test](speedtest.net), store, display results with builtin web server.

---
### Introduction

This strives to be small, efficient all-in-one program (~750 kB statically linked binary including assets) written in pure [rust](https://rust-lang.org).

---
### Features

- tiny list of dependencies: [std](https://doc.rust-lang.org/std/index.html), [libc](https://crates.io/crates/libc), [roxmltree](https://crates.io/crates/roxmltree) + [xmlparser](https://crates.io/crates/xmlparser)
- results are stored in human readable .csv, other storage backends incoming
- builtin web-server to display results in nice [uPlot](https://github.com/leeoniya/uPlot) graphs
- all web assets are included in the program binary
- statically linked with musl - not dependent on system libc, essentially a single binary container
