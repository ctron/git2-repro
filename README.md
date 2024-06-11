# git2 segfault reproducer

**NOTE:** Cloning `https://github.com/CVEProject/cvelistV5.git` might take a while, but it seems to be required to
trigger the issue.

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

## Things to notice

* I tried with a few other repos, but it seems to only fail with `https://github.com/CVEProject/cvelistV5.git` (the
  default).
* Running on `aarch64-unknown-linux-gnu` works in general. But not when compiling with `cross`.
