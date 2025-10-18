# fc

Fast CLI tool that counts files and/or directories using Linux getdents64 (optional recursion, adjustable buffer).

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