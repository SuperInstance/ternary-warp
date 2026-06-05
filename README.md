# ternary-warp

**Value transformations for ternary signals. Clamp, quantize, fold, warp, smooth.**

Raw ternary signals sometimes aren't in the shape you need. This crate provides lightweight transformation functions: clamp to a sub-range, quantize continuous values to ternary, fold pairs with a binary function, warp each value through a mapping, and smooth with a moving-average filter.

No structs, no state, no allocation beyond the output vector. Just pure functions that transform one ternary signal into another.

## What's Inside

- **`clamp(values, min, max)`** — restrict values to a sub-range
- **`quantize(values, thresholds)`** — map continuous f64 to {-1, 0, +1} using (low, high) thresholds
- **`fold(values, f)`** — combine adjacent pairs with a binary function
- **`warp(values, map)`** — apply a per-element mapping function
- **`smooth(values, radius)`** — moving-average smoothing, snap back to ternary

## Quick Example

```rust
use ternary_warp::*;

// Quantize continuous values to ternary
let continuous = vec![0.8, -0.3, 0.1, -0.9];
let ternary = quantize(&continuous, (-0.3, 0.3));
// [1, -1, 0, -1]

// Warp: negate every value
let negated = warp(&[-1, 0, 1], |v| -v);
// [1, 0, -1]

// Smooth with radius 1 (3-element moving average)
let noisy = vec![1, -1, 1, -1, 1];
let smooth_signal = smooth(&noisy, 1);
// Averages reduce the oscillation, snapped back to ternary
```

## The Insight

**Every ternary transformation is a channel.** Clamp is a filter that blocks certain values. Quantize is an analog-to-digital converter. Warp is a lookup table. Fold is a reduction. Smooth is a low-pass filter. These are the atomic operations you compose to build ternary signal processing pipelines.

**Use cases:**
- **Signal preprocessing** — prepare raw signals for ternary processing
- **Data conversion** — quantize continuous sensor data to ternary
- **Pipeline building** — compose clamp → warp → smooth for clean signals
- **Testing** — generate and transform test signals

## See Also

- **ternary-bite** — destructive transformations (crush, fold, rotate)
- **ternary-mixer** — blending multiple signals
- **ternary-transform** — more complex signal transformations

## Install

```bash
cargo add ternary-warp
```

## License

MIT
