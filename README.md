# git2 segfault reproducer

## What typically works

Run on `aarch64-unknown-linux-gnu`:

```bash
cargo run --release -- --path cvelistV5
```

## What typically breaks

```bash
cross build --release --target aarch64-unknown-linux-gnu
```

Then (on an actual `aarch64` target):

```bash
./git2-repro --path cvelistV5
```
