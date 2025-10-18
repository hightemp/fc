# fc

Fast CLI tool that counts files and/or directories using Linux getdents64 (optional recursion, adjustable buffer).

## Usage

Build:
```bash
cargo build --release
```

Run:
```bash
target/release/fc [OPTIONS] [PATH]
```

Options:
- -r, --recursive — Recursively walk subdirectories
- -D, --dirs — Count directories (instead of files)
- -A, --all — Count both files and directories (overrides the default of counting files only)
- -b, --buf-mb <MiB> — Buffer size in MiB for SYS_getdents64 (default: 5)
- PATH — Directory to traverse (default: .)

Examples:
- Count files in current directory: `fc`
- Count directories only: `fc -D`
- Count both files and directories: `fc -A`
- Count files recursively: `fc -r`
- Count directories recursively under /var/log: `fc -rD /var/log`
- Increase buffer to 8 MiB and recurse: `fc -b 8 -r`

Benchmark and test data:
- Use [scripts/gen_bench.sh](scripts/gen_bench.sh) to generate large test sets and benchmark `fc` vs `ls -1U | wc -l`.

## Benching

```console
$ ./scripts/gen_bench.sh 
[1/4] Preparing directory: ./testdir (files: 1000000)
[2/4] Generating files...
xargs: warning: options --max-args and --replace/-I/-i are mutually exclusive, ignoring previous --max-args value
[3/4] Building fc (release)...
    Finished `release` profile [optimized] target(s) in 0.01s
Sanity: fc=1000000, ls|wc=1000000
real=0:00.16 user=0.00 sys=0.15 maxrss=7400KB
real=0:00.26 user=0.09 sys=0.17 maxrss=3456KB
```

## License

MIT

