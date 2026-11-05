# Diagnostic tools

Throwaway tools written while working on FTE/TLF support. They are not part of Ruffle's Cargo workspace nor of the regression test suite.

## `outils/abcdump`

Minimal reader of a SWF's AVM2 bytecode (ABC), built on this repository's `swf` crate: it lists classes, methods and instructions. It was used to check what `spark.components.Label.truncateText` really does in `spark_4.1.0.16076.swf`, which differs from the published Flex sources.

```sh
cd operis/outils/abcdump && cargo run --release -- <file.swf>
```

## `essais/`

Throwaway AS3 tests of FTE line breaking (`fte-coupure-lignes`, `fte-largeur-insuffisante`), compiled by the ASC compiler embedded in the test harness (`[[compilers]]` in `test.toml`).

**`output.txt` is NOT a Flash Player reference**: it is Ruffle's output at the time of the experiment. These folders must not be moved as they are into `tests/tests/swfs/`: a real regression test needs an output produced by Flash Player (see upstream `CONTRIBUTING.md`).

To run one on demand: copy it temporarily into `tests/tests/swfs/avm2/<name>/`, then `cargo test -p tests --test tests -- avm2/<name>`.
