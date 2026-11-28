# Test hygiene

Three roles per evaluation file: **train**, **dev** (look as often as needed), **test** (locked by sha256, scored once per work line).

* `tests/locked/MANIFEST.tsv` lists every `tests/*.tsv` with its sha256 and role. Append-only: an edited file gets a new row, never a changed one.
* `tests/locked/LOOKS.tsv` gets one row per scoring of a `test` file. A second score turns the file into dev and a new test is cut.
* `cargo test --test locked_manifest` fails on an edited locked file, an unlisted `tests/*.tsv`, a manifest line changed since `origin/main`, or a `test` file with two looks. Paths that start with `/` are external sets, hashed when present.
* Rules, list entries, thresholds and per-word switches are chosen on dev only. A test failure is never copied into a table or a training seed.
* Agreement with another G2P is reported as "agreement with <name>", not as accuracy.
