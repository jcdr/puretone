# Audio formulas (shipped)

Constants:

- `MINIMUM_FREQUENCY_HERTZ = 20`
- `MAXIMUM_FREQUENCY_HERTZ = 20000`
- `MINIMUM_AMPLITUDE_DECIBELS = -100`
- `MAXIMUM_AMPLITUDE_DECIBELS = 0`
- `AUDIO_SAMPLE_RATE_HERTZ = 48000`

Log frequency from slider position `t ∈ [0,1]`:

```
f = 20 * (20000/20)^t
```

Midpoint `t = 0.5` is geometric mean `sqrt(20*20000) ≈ 632.5 Hz`.

Linear gain from dB:

```
g = 10^(dB/20)
```

Examples: `0 → 1`, `-20 → 0.1`, `-6 ≈ 0.501`, `-80 → 1e-4`, `-100 → 1e-5`.

Phase advance per sample:

```
phase += 2π * f / 48000
phase = phase % 2π   # when phase >= 2π
```

Sample:

```
x = sin(phase) * g
```
