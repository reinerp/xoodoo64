# Benchmarks of Xoodoo permutation and a 64-bit variant of it

We compare the 32b-word Xoodoo permutation with a 64b-word variant of it that we call Xoodoo64.

## Results

You can run the benchmarks with:

```
<install Rust from https://rustup.rs/>
RUSTFLAGS="-C target-cpu=native" cargo run -r
```

### Benchmark naming conventions

Given a benchmark name like `xoodoo64_neon_sha3_x4` this means:

* The permutation is named either `xoodoo` (the actually specified 32b-word Xoodoo permutation) or `xoodoo64` (a naive variant of it using 64b words).
* The instruction set used is either `scalar` (not using any SIMD instructions), `neon` (using NEON instructions but not SHA3 instructions), or `neon_sha3` (using NEON instructions plus SHA3 extensions).
* The parallelism is either the empty string (only one instance of the permutation), or `_x2` (2 parallel instances of the permutation) or `_x4` (4 parallel instances of the permutation)

In all cases I benchmark the 12-round permutation, i.e. as in Xoodyak rather than Xoofff.

### Throughput case
The relevant subset of results for the highly parallel throughput-sensitive case, as in e.g. Xoofff:

```
name                          throughput (cpb)
xoodoo_scalar_x4                          3.41
xoodoo64_scalar_x4                        2.13
xoodoo_neon_x4                            1.70
xoodoo_neon_sha3_x4                       1.68
xoodoo64_neon_sha3_x4                     1.24
```

The 64b variant is 1.35x the throughput of the 32b variant. This is primarily due
to the 64b variant benefitting from 64b SIMD rotations that are available under the SHA3 CPU extensions.

### Latency case

Relevant results for the completely sequential latency-sensitive case, as in e.g. single-stream Xoodyak:

```
name                               latency (ns)
xoodoo_scalar                             114.2
xoodoo_neon_sha3_x4                        98.7
xoodoo_neon_x4                             98.6
xoodoo64_neon_sha3_x2                      69.0
xoodoo64_scalar                            53.7
```

The 64b variant is 1.83x lower latency than the 32b variant. 

* The best 64b implementation uses scalar instructions, and on Apple M-series CPUs the scalar bitwise
  instructions have latency 1, whereas the SIMD bitwise instructions have latency 2. Hence the improved
  latency of the scalar implementation.
* The best 32b implementation uses SIMD instructions. This is much lower latency because each SIMD instruction
  has 2x the latency of a scalar instruction. The scalar 32b implementation is uncompetitive because it
  requires 2x as many uops, becoming throughput-limited rather than latency-limited.

Perhaps surprisingly, the best latency of SIMD implementations is not achieved by the `_x1` implementations, but instead
by the `_x2` or `_x4` implementations. This is because those implementations switch from parallelism within a permutation to parallelism across permutations, and the latter avoids the need for cross-lane shuffles.


### All results

```
Assuming CPU frequency is 3.5 GHz
name                               latency (ns)   par  throughput (ns) throughput (cpb)
xoodoo_scalar                             114.2    1x             60.6             4.42
xoodoo_scalar_x2                          135.9    2x             47.9             3.49
xoodoo_scalar_x4                          189.5    4x             46.8             3.41
xoodoo64_scalar                            53.7    1x             46.0             3.35
xoodoo64_scalar_x2                         71.4    2x             34.4             2.51
xoodoo64_scalar_x4                        119.1    4x             29.3             2.13
xoodoo_neon                               107.6    1x             49.8             3.63
xoodoo_neon_x2                            103.3    2x             39.3             2.87
xoodoo_neon_x4                             98.6    4x             23.3             1.70
xoodoo_neon_sha3                          107.1    1x             49.7             3.62
xoodoo_neon_sha3_x2                       106.0    2x             40.5             2.95
xoodoo_neon_sha3_x4                        98.7    4x             23.1             1.68
xoodoo64_neon_sha3                         74.2    1x             29.9             2.18
xoodoo64_neon_sha3_x2                      69.0    2x             27.2             1.99
xoodoo64_neon_sha3_x4                      75.4    4x             17.0             1.24
```
