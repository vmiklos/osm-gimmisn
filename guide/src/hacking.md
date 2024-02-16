# Development notes

Even tough Rust comes with its own `cargo` build tool, this project uses some build-time created
caches, so always run `make` after changing the code and before using it.

## Coding style

- Build HTML strings using `yattag`, return a `yattag::Doc` if the result is an escaped string.

- Path handling: make relative paths absolute using `context::Context::get_abspath()`.

This has the benefit that real and test config/data (under `/` and `tests/`) can be separated via a
single parameter to the `context::Context` constructor.

- Try to keep module size under 1000 lines to avoid monster modules:

```bash
for i in $(git ls-files|grep rs$|grep -v tests.rs); do lines=$(wc -l < $i); if [ $lines -gt 1000 ]; then echo "$i is too large: $lines lines"; fi; done
```

- Try to make JS optional. If a link can be handled with and without JS, then generate HTML which
  goes to the no-JS version, then JS can tweak the DOM to invoke JS instead.

## TS debugging

Bundled JS can be minified (for production) and also source maps can be added (for debugging). The
default output is for production, but touching a TS source file and invoking:

```bash
make TSDEBUG=1
```

produces output that is for debugging.

## Rust debugging

`make` defaults to release builds. To switch to a debug build:

```bash
rm -rf target/
echo RSDEBUG=1 > config.mak
make
```

To run a single test:

```bash
RUST_BACKTRACE=1 cargo test --lib -- --exact --nocapture wsgi_json::tests::test_missing_streets_update_result_json
```

Tests follow the [Favor real dependencies for unit
testing](https://stackoverflow.blog/2022/01/03/favor-real-dependencies-for-unit-testing/) pattern,
i.e. apart from filesystem, network or time (see `src/context.rs` for the exact list), no mocking is
used.

Debugging `workdir/stats/stats.json` generation:

```
cargo run -- cron --mode stats --no-overpass
```

## Rust performance profiling

The symbols profile enables debug symbols while keeping optimizations on:

```bash
cargo build --profile symbols
valgrind --tool=callgrind target/symbols/osm-gimmisn missing_housenumbers budapest_11
```

## YAML schema

The YAML schema is meant to provide reference documentation in the long run, so guide/src/usage.md can
focus on tutorial documentation.

```bash
ajv validate -s data/relations.schema.yaml -d data/relations.yaml
for i in data/relation-*.yaml; do ajv validate -s data/relation.schema.yaml -d $i || break; done
```

## Checklist

Ideally CI checks everything before a commit hits master, but here are a few
things which are not part of CI:

- HTML validation: <https://validator.w3.org/nu/?doc=https%3A%2F%2Fosm-gimmisn.vmiklos.hu%2Fosm>

- CSS validation:
  <http://jigsaw.w3.org/css-validator/validator?uri=https%3A%2F%2Fosm-gimmisn.vmiklos.hu%2Fosm%2Fstatic%2Fosm.min.css>

- Run `cargo outdated --depth=1` from time to time and make sure Rust dependencies are reasonably up to date.

- Update `.github/workflows/tests.yml` based on `rustc --version`.

- Run `npm outdated` from time to time and make sure JS dependencies are reasonably up to date.
