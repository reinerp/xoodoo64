use std::hint::black_box;
use std::mem::transmute;
use std::time::Instant;

#[inline]
fn read128(x: &[u8]) -> [u8; 16] {
    let mut result = [0u8; 16];
    result.copy_from_slice(x);
    result
}

#[inline]
fn write128(x: [u8; 16], y: &mut [u8]) {
    y[0..16].copy_from_slice(&x);
}

#[inline]
fn read32(x: &[u8]) -> u32 {
    let mut result = [0u8; 4];
    result.copy_from_slice(&x[0..4]);
    u32::from_le_bytes(result)
}

#[inline]
fn write32(x: u32, y: &mut [u8]) {
    y[0..4].copy_from_slice(&x.to_le_bytes());
}

#[inline]
fn read64(x: &[u8]) -> u64 {
    let mut result = [0u8; 8];
    result.copy_from_slice(&x[0..8]);
    u64::from_le_bytes(result)
}

#[inline]
fn write64(x: u64, y: &mut [u8]) {
    y[0..8].copy_from_slice(&x.to_le_bytes());
}

const ROUND_KEYS: [u32; 12] = [
    0x00000058, 0x00000038, 0x000003C0, 0x000000D0, 0x00000120, 0x00000014, 0x00000060, 0x0000002C,
    0x00000380, 0x000000F0, 0x000001A0, 0x00000012,
];

#[cfg(target_arch = "aarch64")]
#[inline]
fn xoodoo_aarch64(x: &mut [u8; 48]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut a: uint32x4_t = transmute(read128(&x[0..16]));
        let mut b: uint32x4_t = transmute(read128(&x[16..32]));
        let mut c: uint32x4_t = transmute(read128(&x[32..48]));

        let rho_west_1: uint8x16_t =
            transmute([12u8, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let rho_east_2: uint8x16_t =
            transmute([11u8, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p = veorq_u32(veorq_u32(a, b), c);
            p = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p), rho_west_1));
            p = vsliq_n_u32(vshrq_n_u32(p, 32 - 5), p, 5);
            let mut e = p;
            p = vsliq_n_u32(vshrq_n_u32(p, 32 - 14), p, 14);
            e = veorq_u32(e, p);
            a = veorq_u32(a, e);
            b = veorq_u32(b, e);
            c = veorq_u32(c, e);

            // rho west
            b = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b), rho_west_1));
            c = vsliq_n_u32(vshrq_n_u32(c, 32 - 11), c, 11);

            // iota
            let round_const = vdupq_n_u32(0);
            let round_const = vsetq_lane_u32(round_key, round_const, 0);
            a = veorq_u32(a, round_const);

            // chi
            let a2 = veorq_u32(vbicq_u32(c, b), a);
            let b2 = veorq_u32(vbicq_u32(a, c), b);
            let c2 = veorq_u32(vbicq_u32(b, a), c);
            a = a2;
            b = b2;
            c = c2;

            // rho east
            b = vsliq_n_u32(vshrq_n_u32(b, 32 - 1), b, 1);
            c = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c), rho_east_2));
        }

        // Store results back
        write128(transmute(a), &mut x[0..16]);
        write128(transmute(b), &mut x[16..32]);
        write128(transmute(c), &mut x[32..48]);
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline]
unsafe fn xoodoo_aarch64_sha3(x: &mut [u8; 48]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut a: uint32x4_t = transmute(read128(&x[0..16]));
        let mut b: uint32x4_t = transmute(read128(&x[16..32]));
        let mut c: uint32x4_t = transmute(read128(&x[32..48]));

        let rho_west_1: uint8x16_t =
            transmute([12u8, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let rho_east_2: uint8x16_t =
            transmute([11u8, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p = veor3q_u32(a, b, c);
            p = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p), rho_west_1));
            p = vsliq_n_u32(vshrq_n_u32(p, 32 - 5), p, 5);
            let mut e = p;
            p = vsliq_n_u32(vshrq_n_u32(p, 32 - 14), p, 14);
            e = veorq_u32(e, p);
            a = veorq_u32(a, e);
            b = veorq_u32(b, e);
            c = veorq_u32(c, e);

            // rho west
            b = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b), rho_west_1));
            c = vsliq_n_u32(vshrq_n_u32(c, 32 - 11), c, 11);

            // iota
            let round_const = vdupq_n_u32(0);
            let round_const = vsetq_lane_u32(round_key, round_const, 0);
            a = veorq_u32(a, round_const);

            // chi
            let a2 = vbcaxq_u32(c, b, a);
            let b2 = vbcaxq_u32(a, c, b);
            let c2 = vbcaxq_u32(b, a, c);
            a = a2;
            b = b2;
            c = c2;

            // rho east
            b = vsliq_n_u32(vshrq_n_u32(b, 32 - 1), b, 1);
            c = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c), rho_east_2));
        }

        // Store results back
        write128(transmute(a), &mut x[0..16]);
        write128(transmute(b), &mut x[16..32]);
        write128(transmute(c), &mut x[32..48]);
    }
}

#[inline]
fn xoodoo_scalar(state: &mut [u8; 48]) {
    let mut x = [
        read32(&state[0..4]),
        read32(&state[4..8]),
        read32(&state[8..12]),
        read32(&state[12..16]),
        read32(&state[16..20]),
        read32(&state[20..24]),
        read32(&state[24..28]),
        read32(&state[28..32]),
        read32(&state[32..36]),
        read32(&state[36..40]),
        read32(&state[40..44]),
        read32(&state[44..48]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0 = x[0] ^ x[4] ^ x[8];
        let mut p1 = x[1] ^ x[5] ^ x[9];
        let mut p2 = x[2] ^ x[6] ^ x[10];
        let mut p3 = x[3] ^ x[7] ^ x[11];
        p0 = p0.rotate_left(5) ^ p0.rotate_left(14);
        p1 = p1.rotate_left(5) ^ p1.rotate_left(14);
        p2 = p2.rotate_left(5) ^ p2.rotate_left(14);
        p3 = p3.rotate_left(5) ^ p3.rotate_left(14);
        x[0] ^= p0;
        x[4] ^= p0;
        x[8] ^= p0;
        x[1] ^= p1;
        x[5] ^= p1;
        x[9] ^= p1;
        x[2] ^= p2;
        x[6] ^= p2;
        x[10] ^= p2;
        x[3] ^= p3;
        x[7] ^= p3;
        x[11] ^= p3;

        // rho west
        (x[4], x[5], x[6], x[7]) = (x[7], x[4], x[5], x[6]);
        x[8] = x[8].rotate_left(11);
        x[9] = x[9].rotate_left(11);
        x[10] = x[10].rotate_left(11);
        x[11] = x[11].rotate_left(11);

        // iota
        x[0] ^= round_key;

        // chi
        let t0 = x[0] ^ (!x[4] & x[8]);
        let t1 = x[4] ^ (!x[8] & x[0]);
        let t2 = x[8] ^ (!x[0] & x[4]);
        x[0] = t0;
        x[4] = t1;
        x[8] = t2;
        let t0 = x[1] ^ (!x[5] & x[9]);
        let t1 = x[5] ^ (!x[9] & x[1]);
        let t2 = x[9] ^ (!x[1] & x[5]);
        x[1] = t0;
        x[5] = t1;
        x[9] = t2;
        let t0 = x[2] ^ (!x[6] & x[10]);
        let t1 = x[6] ^ (!x[10] & x[2]);
        let t2 = x[10] ^ (!x[2] & x[6]);
        x[2] = t0;
        x[6] = t1;
        x[10] = t2;
        let t0 = x[3] ^ (!x[7] & x[11]);
        let t1 = x[7] ^ (!x[11] & x[3]);
        let t2 = x[11] ^ (!x[3] & x[7]);
        x[3] = t0;
        x[7] = t1;
        x[11] = t2;

        // rho east
        x[4] = x[4].rotate_left(1);
        x[5] = x[5].rotate_left(1);
        x[6] = x[6].rotate_left(1);
        x[7] = x[7].rotate_left(1);
        (x[8], x[9], x[10], x[11]) = (x[10], x[11], x[8], x[9]);
        x[8] = x[8].rotate_left(8);
        x[9] = x[9].rotate_left(8);
        x[10] = x[10].rotate_left(8);
        x[11] = x[11].rotate_left(8);
    }

    write32(x[0], &mut state[0..4]);
    write32(x[1], &mut state[4..8]);
    write32(x[2], &mut state[8..12]);
    write32(x[3], &mut state[12..16]);
    write32(x[4], &mut state[16..20]);
    write32(x[5], &mut state[20..24]);
    write32(x[6], &mut state[24..28]);
    write32(x[7], &mut state[28..32]);
    write32(x[8], &mut state[32..36]);
    write32(x[9], &mut state[36..40]);
    write32(x[10], &mut state[40..44]);
    write32(x[11], &mut state[44..48]);
}

/// A variant of Xoodoo that is based on 64-bit words.
#[inline]
fn xoodoo64_scalar(state: &mut [u8; 48]) {
    let mut x = [
        read64(&state[0..8]),
        read64(&state[8..16]),
        read64(&state[16..24]),
        read64(&state[24..32]),
        read64(&state[32..40]),
        read64(&state[40..48]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0 = x[0] ^ x[2] ^ x[4];
        let mut p1 = x[1] ^ x[3] ^ x[5];
        p0 = p0.rotate_left(10) ^ p0.rotate_left(29);
        p1 = p1.rotate_left(10) ^ p1.rotate_left(29);
        (p0, p1) = (p1, p0);

        x[0] ^= p0;
        x[2] ^= p0;
        x[4] ^= p0;
        x[1] ^= p1;
        x[3] ^= p1;
        x[5] ^= p1;

        // rho west
        x.swap(2, 3);
        x[4] = x[4].rotate_left(23);
        x[5] = x[5].rotate_left(23);

        // iota
        x[0] ^= round_key as u64;

        // chi
        let t0 = x[0] ^ (!x[2] & x[4]);
        let t1 = x[2] ^ (!x[4] & x[0]);
        let t2 = x[4] ^ (!x[0] & x[2]);
        x[0] = t0;
        x[2] = t1;
        x[4] = t2;
        let t0 = x[1] ^ (!x[3] & x[5]);
        let t1 = x[3] ^ (!x[5] & x[1]);
        let t2 = x[5] ^ (!x[1] & x[3]);
        x[1] = t0;
        x[3] = t1;
        x[5] = t2;

        // rho east
        x[2] = x[2].rotate_left(1);
        x[3] = x[3].rotate_left(1);
        x.swap(4, 5);
        x[4] = x[4].rotate_left(16);
        x[5] = x[5].rotate_left(16);
    }

    write64(x[0], &mut state[0..8]);
    write64(x[1], &mut state[8..16]);
    write64(x[2], &mut state[16..24]);
    write64(x[3], &mut state[24..32]);
    write64(x[4], &mut state[32..40]);
    write64(x[5], &mut state[40..48]);
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline]
unsafe fn xoodoo64_aarch64_sha3(x: &mut [u8; 48]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut a: uint64x2_t = transmute(read128(&x[0..16]));
        let mut b: uint64x2_t = transmute(read128(&x[16..32]));
        let mut c: uint64x2_t = transmute(read128(&x[32..48]));

        let swap_u64s: uint8x16_t =
            transmute([8u8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]);
        let zero: uint64x2_t = transmute([0u8; 16]);
        let rho_east: uint8x16_t =
            transmute([10u8, 11, 12, 13, 14, 15, 8, 9, 2, 3, 4, 5, 6, 7, 0, 1]);

        for &round_key in &ROUND_KEYS {
            // theta
            let p = veor3q_u64(a, b, c);
            let p = vreinterpretq_u64_u8(vqtbl1q_u8(vreinterpretq_u8_u64(p), swap_u64s));
            let tp = vxarq_u64::<19>(p, zero);
            let e = vxarq_u64::<10>(p, tp);
            a = veorq_u64(a, e);
            b = veorq_u64(b, e);
            c = veorq_u64(c, e);

            // rho west
            b = vreinterpretq_u64_u8(vqtbl1q_u8(vreinterpretq_u8_u64(b), swap_u64s));
            c = vxarq_u64::<23>(c, zero);

            // iota
            let round_const = vsetq_lane_u64(round_key as u64, zero, 0);
            a = veorq_u64(a, round_const);

            // chi
            let a2 = vbcaxq_u64(c, b, a);
            let b2 = vbcaxq_u64(a, c, b);
            let c2 = vbcaxq_u64(b, a, c);
            a = a2;
            b = b2;
            c = c2;

            // rho east
            b = vxarq_u64::<1>(b, zero);
            c = vreinterpretq_u64_u8(vqtbl1q_u8(vreinterpretq_u8_u64(c), rho_east));
        }

        // Store results back
        write128(transmute(a), &mut x[0..16]);
        write128(transmute(b), &mut x[16..32]);
        write128(transmute(c), &mut x[32..48]);
    }
}

#[inline(never)]
fn benchmark<const N: usize>(name: &str, f: impl Fn(&mut [u8; N])) {
    const ITERS: usize = 10_000_000;
    // Latency benchmark. Run rounds consecutively, with dependencies between rounds.
    //
    // An approximation of single-stream Xoodyak.
    let mut x = black_box([0u8; N]);
    let start = Instant::now();
    for _ in 0..ITERS {
        f(&mut x);
    }
    black_box(x);
    let elapsed = start.elapsed();
    let latency = elapsed.as_nanos() as f64 / ITERS as f64;
    // Throughput benchmark. Run many independent rounds.
    let start = Instant::now();
    for _ in 0..ITERS {
        let mut x = black_box([0u8; N]);
        f(&mut x);
        black_box(&x);
    }
    let elapsed = start.elapsed();
    let throughput = elapsed.as_nanos() as f64 / ITERS as f64;
    println!("{:<30} {:>16.1} {:16.1}", name, latency, throughput);
}

fn main() {
    println!(
        "{:<30} {:>16} {:>16}",
        "name", "latency (ns)", "throughput (ns)"
    );
    benchmark("xoodoo_scalar", xoodoo_scalar);
    benchmark("xoodoo64_scalar", xoodoo64_scalar);
    benchmark("xoodoo_aarch64", xoodoo_aarch64);
    benchmark("xoodoo_aarch64_sha3", |x| unsafe { xoodoo_aarch64_sha3(x) });
    benchmark("xoodoo64_aarch64_sha3", |x| unsafe {
        xoodoo64_aarch64_sha3(x)
    });
}
