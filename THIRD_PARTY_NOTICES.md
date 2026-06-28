# Third-Party Notices

papur is distributed under the [MIT License](LICENSE). Its release binaries
statically link third-party Rust crates, each under a permissive license (MIT,
BSD-2-Clause, BSD-3-Clause, ISC, Apache-2.0, or Unicode-3.0), per the dependency
policy in [`AGENTS.md`](AGENTS.md) (Boundaries).

## Direct dependencies

| Crate | Version | License | Relied on under |
| --- | --- | --- | --- |
| clap | 4.x | MIT OR Apache-2.0 | MIT |
| indexmap | 2.x | Apache-2.0 OR MIT | MIT |
| miette | 7.x | Apache-2.0 | Apache-2.0 |
| thiserror | 2.x | MIT OR Apache-2.0 | MIT |
| yaml-rust2 | 0.10.x | MIT OR Apache-2.0 | MIT |
| insta (dev only) | 1.x | Apache-2.0 | Apache-2.0 |

`insta` is a development/test dependency and is not part of any distributed
binary, so it carries no distribution obligation.

## Apache-2.0 components

`miette` (with its compile-time macro crate `miette-derive`) is the only runtime
dependency offered solely under Apache-2.0; it is used under the terms of the
Apache License, Version 2.0. As of this writing **no dependency in the tree
ships an upstream `NOTICE` file**, so Apache-2.0 §4(d) imposes no additional
attribution text beyond this acknowledgement.

## Complete bundle

The table above lists direct dependencies for quick review. Release artifacts
should ship a complete, generated attribution bundle covering the full
transitive tree — produced with a tool such as `cargo about` or
`cargo bundle-licenses` — regenerated per release so it never drifts from
`Cargo.lock`.
