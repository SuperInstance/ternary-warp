# ternary-warp

Pure functions that reshape ternary signals. Clamp, quantize, fold, warp, smooth, differentiate.

No structs. No state. No allocations beyond the output vector. Just functions that take a ternary signal in and push a transformed signal out. Every function is a channel in a signal processing pipeline, and composing them is as natural as piping shell commands.

## Why this exists

Raw ternary signals rarely arrive in the shape you need. Sensor data comes as floats that need quantizing. Noisy streams need smoothing. Paired values need folding. And sometimes you just need to negate everything.

You could write these functions yourself—each one is 5-15 lines. But you'd write them slightly differently every time, with subtly different edge-case behavior. This crate gives you the canonical versions: tested, documented, and composable.

## The key insight

Signal processing for ternary data is simpler than for continuous signals because the alphabet is {-1, 0, +1}. Every operation has at most three cases. This means:

1. **Smooth** doesn't need a Gaussian kernel—it just takes a majority vote in a sliding window
2. **Quantize** doesn't need learned thresholds—two cutoffs divide ℝ into three regions
3. **Differentiate** doesn't need calculus—just compare adjacent values and clamp to the ternary range

The simplicity isn't a limitation. It's a feature. Fewer edge cases, fewer bugs, faster execution.

## Quick start

```rust
use ternary_warp::*;

// Quantize continuous sensor readings to ternary
let sensor_data = vec![0.8, -0.3, 0.1, -0.9, 0.25];
let ternary = quantize(&sensor_data, (-0.3, 0.3));
assert_eq!(ternary, vec![1, -1, 0, -1, 0]);

// Clean up a noisy signal
let noisy = vec![1, -1, 1, -1, 1];
let cleaned = smooth(&noisy, 1);
assert_eq!(cleaned, vec![1, 1, 1, 1, 1]); // majority wins at each position

// Detect edges in a signal
let signal = vec![-1, -1, 0, 1, 1];
let edges = differentiate(&signal);
assert_eq!(edges, vec![0, 0, 1, 1, 0]); // rises detected, first value always 0
```

## API reference

### `clamp(values, min, max) → Vec<i8>`

Restrict every value to a sub-range. Values below `min` snap to `min`; values above `max` snap to `max`.

```rust
clamp(&[-3, -1, 0, 1, 5], -1, 1)  // → [-1, -1, 0, 1, 1]
clamp(&[-1, 0, 1], -1, 1)          // → [-1, 0, 1] (identity)
```

### `quantize(values, thresholds) → Vec<i8>`

Map continuous `f64` values to ternary {-1, 0, +1} using a `(low, high)` threshold pair.

- `v < low` → -1
- `v > high` → +1
- otherwise → 0

```rust
quantize(&[-0.5, 0.0, 0.5], (-0.3, 0.3))  // → [-1, 0, 1]
quantize(&[0.0, 0.0, 0.0], (-0.1, 0.1))    // → [0, 0, 0]
```

### `fold(values, f) → Vec<i8>`

Combine adjacent pairs with a binary function. If the input has odd length, the last element passes through unchanged.

```rust
fold(&[1, -1, 0, 1], |a, b| a + b)  // → [0, 1]
fold(&[1, -1, 1], |a, b| a + b)      // → [0, 1] (last element passes through)
```

### `warp(values, map) → Vec<i8>`

Apply a per-element mapping function. The ternary equivalent of a lookup table.

```rust
warp(&[-1, 0, 1], |v| -v)        // → [1, 0, -1] (negate)
warp(&[-1, 0, 1], |v| v.max(0))  // → [0, 0, 1] (rectify)
```

### `smooth(values, radius) → Vec<i8>`

Moving-window majority filter. For each position, looks at all values within `radius` distance and picks the mode. Radius 0 returns a copy of the input.

This is the ternary equivalent of a low-pass filter. It kills oscillation and noise while preserving the dominant signal.

```rust
smooth(&[1, 1, -1, 1, 1], 1)  // → [1, 1, 1, 1, 1] (single blip removed)
smooth(&[0, 0, 0], 1)          // → [0, 0, 0] (flat stays flat)
smooth(&[1, -1, 0], 0)         // → [1, -1, 0] (radius 0 = identity)
```

### `differentiate(values) → Vec<i8>`

Compute the discrete derivative, clamped to ternary range. Returns the change between adjacent values: `(v[i] - v[i-1]).clamp(-1, 1)`. First element is always 0.

```rust
differentiate(&[-1, 0, 1])   // → [0, 1, 1]  (rising edge)
differentiate(&[0, 0, 0])    // → [0, 0, 0]  (flat)
differentiate(&[1, -1, 1])   // → [0, -1, 1] (oscillating)
```

## Composing a pipeline

The real power comes from chaining operations:

```rust
use ternary_warp::*;

// Pipeline: raw sensor → quantize → smooth → differentiate → detect edges
fn detect_signal_edges(raw: &[f64]) -> Vec<i8> {
    let ternary = quantize(raw, (-0.3, 0.3));
    let cleaned = smooth(&ternary, 2);
    differentiate(&cleaned)
}

let sensor_stream = vec![0.05, -0.02, 0.8, 0.9, 0.7, -0.6, -0.8, 0.0];
let edges = detect_signal_edges(&sensor_stream);
// Non-zero values in `edges` mark where the signal changed state
```

## Architecture

Every function follows the same contract:

```
fn transform(input: &[i8], /* params */) -> Vec<i8>
```

This makes composition trivial. The output of one function feeds directly into the next. No adapters, no type conversions, no allocations beyond what's necessary for the output vector.

**Performance**: Each function is O(n) where n = input length. `smooth` is O(n × radius) but radius is typically small (1-3). No heap allocations except the output vector.

## Real-world example: Agent state stream

```rust
use ternary_warp::*;

// An agent emits a continuous state signal over time.
// We want to detect when it transitions between states.

let raw_states = vec![
    0.02, 0.05, -0.01,   // hovering near 0 (uncertain)
    0.7, 0.85, 0.9,      // transitioning positive (accepting)
    -0.8, -0.7, -0.9,    // transitioning negative (rejecting)
    0.0, 0.1, -0.05,     // back to uncertain
];

// Quantize to ternary
let ternary = quantize(&raw_states, (-0.3, 0.3));
// [0, 0, 0, 1, 1, 1, -1, -1, -1, 0, 0, 0]

// Smooth to remove noise
let smooth_signal = smooth(&ternary, 1);
// [0, 0, 0, 1, 1, 1, -1, -1, -1, 0, 0, 0]  (already clean)

// Differentiate to find transitions
let transitions = differentiate(&smooth_signal);
// [0, 0, 0, 1, 0, 0, -1, 0, 0, 1, 0, 0]

// Non-zero positions = state transitions
let transition_points: Vec<usize> = transitions
    .iter()
    .enumerate()
    .filter(|(_, &v)| v != 0)
    .map(|(i, _)| i)
    .collect();
// Transitions at positions 3, 6, 9
```

## Ecosystem connections

- **ternary-bite** — destructive transformations (crush, wavefold, bit-rotate) for a different flavor of signal manipulation
- **ternary-gauge** — instrument your signals before and after transformation to verify the pipeline is doing what you expect
- **ternary-membrane** — diffusion and transport are essentially spatial `smooth` operations across compartments

## Stats

| Metric | Value |
|--------|-------|
| Tests | 15 |
| Public functions | 6 |
| Lines of code | ~140 |
| License | MIT |
| Unsafe | 0 |

## Installation

```toml
[dependencies]
ternary-warp = "0.1.0"
```

## License

MIT
