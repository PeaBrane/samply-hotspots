# samply-hotspots

Fast command-line summaries for `samply record --save-only` profiles.

`samply-hotspots` reads Firefox Profiler JSON emitted by
[`samply`](https://github.com/mstange/samply), walks the sampled stacks, resolves
native addresses with `atos`, and prints a compact hot-function breakdown. It is
intended for the workflow where you want a quick terminal answer before opening
the full Firefox Profiler UI.

The original motivation was profiling Rust binaries on macOS. Firefox Profiler
JSON is rich, but scripting over it in Python gets slow once profiles contain
many threads and stacks. This tool keeps the same basic reporting shape while
moving stack walking and aggregation into Rust and batching `atos` calls.

## What It Reports

- `SELF` time: leaf frame, or the nearest frame belonging to the requested
  target binary in single-image mode.
- `TOTAL` time: inclusive count across every sampled stack frame.
- Optional `--cpu-weighted` output: weights samples by `threadCPUDelta` when the
  profile contains aligned CPU delta data.
- Optional line-level drill-down: resolves sampled addresses for function-name
  substrings passed with `--targets`.
- Mixed-image mode: resolves each sampled frame against its owning image, which
  is useful for profiles of `python -m ...` with Rust extensions or other
  multi-library processes.

This is not a replacement for the Firefox Profiler UI. Use it as a fast first
pass to answer "what is hot enough to inspect next?"

## Requirements

- macOS
- `atos` from Xcode command line tools
- A `samply` profile captured with `--save-only`
- A binary and matching dSYM for single-image Rust symbolication

For Rust binaries, build with debug info and run `dsymutil` before parsing the
profile. For example:

```bash
cargo build --profile profiling -p dynamo-bench --bin offline_replay_bench
dsymutil target/profiling/offline_replay_bench
samply record --save-only -o /tmp/offline_replay_bench_profile.json \
  target/profiling/offline_replay_bench ...
```

## Build

```bash
cargo build --release
```

The binary will be at:

```bash
target/release/samply-hotspots
```

## Usage

Single Rust binary:

```bash
target/release/samply-hotspots \
  --profile /tmp/offline_replay_bench_profile.json \
  --binary /path/to/binary \
  --base 0x100000000 \
  --top 25
```

Mixed Python/Rust-extension or multi-image capture:

```bash
target/release/samply-hotspots \
  --profile /tmp/profile.json \
  --mixed \
  --top 25
```

Line-level drill-down for selected function substrings:

```bash
target/release/samply-hotspots \
  --profile /tmp/profile.json \
  --binary /path/to/binary \
  --targets 'my_crate::hot_function' 'other_function'
```

CPU-weighted output:

```bash
target/release/samply-hotspots \
  --profile /tmp/profile.json \
  --binary /path/to/binary \
  --cpu-weighted
```

## Notes

- In single-image mode, the profile's sampled frame names are treated as Mach-O
  file offsets. The tool adds `--base` before calling `atos`.
- The default base is `0x100000000`, which is typical for arm64 macOS binaries.
- If a frame cannot be symbolicated, the raw frame name or address is kept in
  the output.
- Inclusive percentages can exceed 100% when samples come from multiple threads.
- `--cpu-weighted` is only as precise as the profiler's CPU delta data and the
  sampled stack captured at that point in time.
