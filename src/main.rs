use std::hint::black_box;
use std::mem::transmute;
use std::time::Instant;

#[inline(always)]
fn read128(x: &[u8]) -> [u8; 16] {
    let mut result = [0u8; 16];
    result.copy_from_slice(x);
    result
}

#[inline(always)]
fn write128(x: [u8; 16], y: &mut [u8]) {
    y[0..16].copy_from_slice(&x);
}

#[inline(always)]
fn read32(x: &[u8]) -> u32 {
    let mut result = [0u8; 4];
    result.copy_from_slice(&x[0..4]);
    u32::from_le_bytes(result)
}

#[inline(always)]
fn write32(x: u32, y: &mut [u8]) {
    y[0..4].copy_from_slice(&x.to_le_bytes());
}

#[inline(always)]
fn read64(x: &[u8]) -> u64 {
    let mut result = [0u8; 8];
    result.copy_from_slice(&x[0..8]);
    u64::from_le_bytes(result)
}

#[inline(always)]
fn write64(x: u64, y: &mut [u8]) {
    y[0..8].copy_from_slice(&x.to_le_bytes());
}

const ROUND_KEYS: [u32; 12] = [
    0x00000058, 0x00000038, 0x000003C0, 0x000000D0, 0x00000120, 0x00000014, 0x00000060, 0x0000002C,
    0x00000380, 0x000000F0, 0x000001A0, 0x00000012,
];

#[cfg(target_arch = "aarch64")]
#[inline(never)]
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
#[inline(never)]
fn xoodoo_aarch64_x2(x: &mut [u8; 96]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut a0: uint32x4_t = transmute(read128(&x[0..16]));
        let mut b0: uint32x4_t = transmute(read128(&x[16..32]));
        let mut c0: uint32x4_t = transmute(read128(&x[32..48]));
        let mut a1: uint32x4_t = transmute(read128(&x[48..64]));
        let mut b1: uint32x4_t = transmute(read128(&x[64..80]));
        let mut c1: uint32x4_t = transmute(read128(&x[80..96]));

        let rho_west_1: uint8x16_t =
            transmute([12u8, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let rho_east_2: uint8x16_t =
            transmute([11u8, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0 = veorq_u32(veorq_u32(a0, b0), c0);
            let mut p1 = veorq_u32(veorq_u32(a1, b1), c1);
            p0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p0), rho_west_1));
            p1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p1), rho_west_1));
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 5), p0, 5);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 5), p1, 5);
            let mut e0 = p0;
            let mut e1 = p1;
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 14), p0, 14);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 14), p1, 14);
            e0 = veorq_u32(e0, p0);
            e1 = veorq_u32(e1, p1);
            a0 = veorq_u32(a0, e0);
            a1 = veorq_u32(a1, e1);
            b0 = veorq_u32(b0, e0);
            b1 = veorq_u32(b1, e1);
            c0 = veorq_u32(c0, e0);
            c1 = veorq_u32(c1, e1);

            // rho west
            b0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b0), rho_west_1));
            b1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b1), rho_west_1));
            c0 = vsliq_n_u32(vshrq_n_u32(c0, 32 - 11), c0, 11);
            c1 = vsliq_n_u32(vshrq_n_u32(c1, 32 - 11), c1, 11);

            // iota
            let round_const = vdupq_n_u32(0);
            let round_const = vsetq_lane_u32(round_key, round_const, 0);
            a0 = veorq_u32(a0, round_const);
            a1 = veorq_u32(a1, round_const);

            // chi
            let t0 = veorq_u32(vbicq_u32(c0, b0), a0);
            let t1 = veorq_u32(vbicq_u32(a0, c0), b0);
            let t2 = veorq_u32(vbicq_u32(b0, a0), c0);
            a0 = t0;
            b0 = t1;
            c0 = t2;
            let t0 = veorq_u32(vbicq_u32(c1, b1), a1);
            let t1 = veorq_u32(vbicq_u32(a1, c1), b1);
            let t2 = veorq_u32(vbicq_u32(b1, a1), c1);
            a1 = t0;
            b1 = t1;
            c1 = t2;

            // rho east
            b0 = vsliq_n_u32(vshrq_n_u32(b0, 32 - 1), b0, 1);
            b1 = vsliq_n_u32(vshrq_n_u32(b1, 32 - 1), b1, 1);
            c0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c0), rho_east_2));
            c1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c1), rho_east_2));
        }

        // Store results back
        write128(transmute(a0), &mut x[0..16]);
        write128(transmute(b0), &mut x[16..32]);
        write128(transmute(c0), &mut x[32..48]);
        write128(transmute(a1), &mut x[48..64]);
        write128(transmute(b1), &mut x[64..80]);
        write128(transmute(c1), &mut x[80..96]);
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(never)]
fn xoodoo_aarch64_x4(state: &mut [u8; 192]) {
    use std::arch::aarch64::*;

    unsafe {
        let x: [uint64x2_t; 12] = [
            transmute(read128(&state[0..16])),
            transmute(read128(&state[16..32])),
            transmute(read128(&state[32..48])),
            transmute(read128(&state[48..64])),
            transmute(read128(&state[64..80])),
            transmute(read128(&state[80..96])),
            transmute(read128(&state[96..112])),
            transmute(read128(&state[112..128])),
            transmute(read128(&state[128..144])),
            transmute(read128(&state[144..160])),
            transmute(read128(&state[160..176])),
            transmute(read128(&state[176..192])),
        ];
        let mut x: [uint32x4_t; 12] = [
            transmute(vtrn1q_u64(x[0], x[6])),
            transmute(vtrn1q_u64(x[1], x[7])),
            transmute(vtrn1q_u64(x[2], x[8])),
            transmute(vtrn1q_u64(x[3], x[9])),
            transmute(vtrn1q_u64(x[4], x[10])),
            transmute(vtrn1q_u64(x[5], x[11])),
            transmute(vtrn2q_u64(x[0], x[6])),
            transmute(vtrn2q_u64(x[1], x[7])),
            transmute(vtrn2q_u64(x[2], x[8])),
            transmute(vtrn2q_u64(x[3], x[9])),
            transmute(vtrn2q_u64(x[4], x[10])),
            transmute(vtrn2q_u64(x[5], x[11])),
        ];
        x = [
            vtrn1q_u32(x[0], x[3]),
            vtrn1q_u32(x[1], x[4]),
            vtrn1q_u32(x[2], x[5]),
            vtrn2q_u32(x[0], x[3]),
            vtrn2q_u32(x[1], x[4]),
            vtrn2q_u32(x[2], x[5]),
            vtrn1q_u32(x[6], x[9]),
            vtrn1q_u32(x[7], x[10]),
            vtrn1q_u32(x[8], x[11]),
            vtrn2q_u32(x[6], x[9]),
            vtrn2q_u32(x[7], x[10]),
            vtrn2q_u32(x[8], x[11]),
        ];

        let rho_east_2: uint8x16_t =
            transmute([3u8, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0 = veorq_u32(x[0], veorq_u32(x[4], x[8]));
            let mut p1 = veorq_u32(x[1], veorq_u32(x[5], x[9]));
            let mut p2 = veorq_u32(x[2], veorq_u32(x[6], x[10]));
            let mut p3 = veorq_u32(x[3], veorq_u32(x[7], x[11]));
            let mut e0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 5), p0, 5);
            let mut e1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 5), p1, 5);
            let mut e2 = vsliq_n_u32(vshrq_n_u32(p2, 32 - 5), p2, 5);
            let mut e3 = vsliq_n_u32(vshrq_n_u32(p3, 32 - 5), p3, 5);
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 14), p0, 14);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 14), p1, 14);
            p2 = vsliq_n_u32(vshrq_n_u32(p2, 32 - 14), p2, 14);
            p3 = vsliq_n_u32(vshrq_n_u32(p3, 32 - 14), p3, 14);
            e0 = veorq_u32(e0, p0);
            e1 = veorq_u32(e1, p1);
            e2 = veorq_u32(e2, p2);
            e3 = veorq_u32(e3, p3);
            (e0, e1, e2, e3) = (e1, e2, e3, e0);
            x[0] = veorq_u32(x[0], e0);
            x[4] = veorq_u32(x[4], e0);
            x[8] = veorq_u32(x[8], e0);
            x[1] = veorq_u32(x[1], e1);
            x[5] = veorq_u32(x[5], e1);
            x[9] = veorq_u32(x[9], e1);
            x[2] = veorq_u32(x[2], e2);
            x[6] = veorq_u32(x[6], e2);
            x[10] = veorq_u32(x[10], e2);
            x[3] = veorq_u32(x[3], e3);
            x[7] = veorq_u32(x[7], e3);
            x[11] = veorq_u32(x[11], e3);

            // rho west
            (x[4], x[5], x[6], x[7]) = (x[7], x[4], x[5], x[6]);
            x[8] = vsliq_n_u32(vshrq_n_u32(x[8], 32 - 11), x[8], 11);
            x[9] = vsliq_n_u32(vshrq_n_u32(x[9], 32 - 11), x[9], 11);
            x[10] = vsliq_n_u32(vshrq_n_u32(x[10], 32 - 11), x[10], 11);
            x[11] = vsliq_n_u32(vshrq_n_u32(x[11], 32 - 11), x[11], 11);

            // iota
            let round_key = vdupq_n_u32(round_key);
            x[0] = veorq_u32(x[0], round_key);

            // chi
            let t0 = veorq_u32(vbicq_u32(x[0], x[4]), x[8]);
            let t1 = veorq_u32(vbicq_u32(x[4], x[8]), x[0]);
            let t2 = veorq_u32(vbicq_u32(x[8], x[0]), x[4]);
            x[0] = t0;
            x[4] = t1;
            x[8] = t2;
            let t0 = veorq_u32(vbicq_u32(x[1], x[5]), x[9]);
            let t1 = veorq_u32(vbicq_u32(x[5], x[9]), x[1]);
            let t2 = veorq_u32(vbicq_u32(x[9], x[1]), x[5]);
            x[1] = t0;
            x[5] = t1;
            x[9] = t2;
            let t0 = veorq_u32(vbicq_u32(x[2], x[6]), x[10]);
            let t1 = veorq_u32(vbicq_u32(x[6], x[10]), x[2]);
            let t2 = veorq_u32(vbicq_u32(x[10], x[2]), x[6]);
            x[2] = t0;
            x[6] = t1;
            x[10] = t2;
            let t0 = veorq_u32(vbicq_u32(x[3], x[7]), x[11]);
            let t1 = veorq_u32(vbicq_u32(x[7], x[11]), x[3]);
            let t2 = veorq_u32(vbicq_u32(x[11], x[3]), x[7]);
            x[3] = t0;
            x[7] = t1;
            x[11] = t2;

            // rho east
            x[4] = vsliq_n_u32(vshrq_n_u32(x[4], 32 - 1), x[4], 1);
            x[5] = vsliq_n_u32(vshrq_n_u32(x[5], 32 - 1), x[5], 1);
            x[6] = vsliq_n_u32(vshrq_n_u32(x[6], 32 - 1), x[6], 1);
            x[7] = vsliq_n_u32(vshrq_n_u32(x[7], 32 - 1), x[7], 1);
            (x[8], x[9], x[10], x[11]) = (x[10], x[11], x[8], x[9]);
            x[8] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[8]), rho_east_2));
            x[9] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[9]), rho_east_2));
            x[10] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[10]), rho_east_2));
            x[11] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[11]), rho_east_2));
        }

        let mut x: [uint64x2_t; 12] = [
            transmute(vtrn1q_u32(x[0], x[3])),
            transmute(vtrn1q_u32(x[1], x[4])),
            transmute(vtrn1q_u32(x[2], x[5])),
            transmute(vtrn2q_u32(x[0], x[3])),
            transmute(vtrn2q_u32(x[1], x[4])),
            transmute(vtrn2q_u32(x[2], x[5])),
            transmute(vtrn1q_u32(x[6], x[9])),
            transmute(vtrn1q_u32(x[7], x[10])),
            transmute(vtrn1q_u32(x[8], x[11])),
            transmute(vtrn2q_u32(x[6], x[9])),
            transmute(vtrn2q_u32(x[7], x[10])),
            transmute(vtrn2q_u32(x[8], x[11])),
        ];
        x = [
            vtrn1q_u64(x[0], x[6]),
            vtrn1q_u64(x[1], x[7]),
            vtrn1q_u64(x[2], x[8]),
            vtrn1q_u64(x[3], x[9]),
            vtrn1q_u64(x[4], x[10]),
            vtrn1q_u64(x[5], x[11]),
            vtrn2q_u64(x[0], x[6]),
            vtrn2q_u64(x[1], x[7]),
            vtrn2q_u64(x[2], x[8]),
            vtrn2q_u64(x[3], x[9]),
            vtrn2q_u64(x[4], x[10]),
            vtrn2q_u64(x[5], x[11]),
        ];

        // Store results back
        write128(transmute(x[0]), &mut state[0..16]);
        write128(transmute(x[1]), &mut state[16..32]);
        write128(transmute(x[2]), &mut state[32..48]);
        write128(transmute(x[3]), &mut state[48..64]);
        write128(transmute(x[4]), &mut state[64..80]);
        write128(transmute(x[5]), &mut state[80..96]);
        write128(transmute(x[6]), &mut state[96..112]);
        write128(transmute(x[7]), &mut state[112..128]);
        write128(transmute(x[8]), &mut state[128..144]);
        write128(transmute(x[9]), &mut state[144..160]);
        write128(transmute(x[10]), &mut state[160..176]);
        write128(transmute(x[11]), &mut state[176..192]);
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
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

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
unsafe fn xoodoo_aarch64_sha3_x2(x: &mut [u8; 96]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut a0: uint32x4_t = transmute(read128(&x[0..16]));
        let mut b0: uint32x4_t = transmute(read128(&x[16..32]));
        let mut c0: uint32x4_t = transmute(read128(&x[32..48]));
        let mut a1: uint32x4_t = transmute(read128(&x[48..64]));
        let mut b1: uint32x4_t = transmute(read128(&x[64..80]));
        let mut c1: uint32x4_t = transmute(read128(&x[80..96]));

        let rho_west_1: uint8x16_t =
            transmute([12u8, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let rho_east_2: uint8x16_t =
            transmute([11u8, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0 = veor3q_u32(a0, b0, c0);
            let mut p1 = veor3q_u32(a1, b1, c1);
            p0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p0), rho_west_1));
            p1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(p1), rho_west_1));
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 5), p0, 5);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 5), p1, 5);
            let mut e0 = p0;
            let mut e1 = p1;
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 14), p0, 14);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 14), p1, 14);
            e0 = veorq_u32(e0, p0);
            e1 = veorq_u32(e1, p1);
            a0 = veorq_u32(a0, e0);
            a1 = veorq_u32(a1, e1);
            b0 = veorq_u32(b0, e0);
            b1 = veorq_u32(b1, e1);
            c0 = veorq_u32(c0, e0);
            c1 = veorq_u32(c1, e1);

            // rho west
            b0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b0), rho_west_1));
            b1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(b1), rho_west_1));
            c0 = vsliq_n_u32(vshrq_n_u32(c0, 32 - 11), c0, 11);
            c1 = vsliq_n_u32(vshrq_n_u32(c1, 32 - 11), c1, 11);

            // iota
            let round_const = vdupq_n_u32(0);
            let round_const = vsetq_lane_u32(round_key, round_const, 0);
            a0 = veorq_u32(a0, round_const);
            a1 = veorq_u32(a1, round_const);

            // chi
            let t0 = vbcaxq_u32(c0, b0, a0);
            let t1 = vbcaxq_u32(a0, c0, b0);
            let t2 = vbcaxq_u32(b0, a0, c0);
            a0 = t0;
            b0 = t1;
            c0 = t2;
            let t0 = vbcaxq_u32(c1, b1, a1);
            let t1 = vbcaxq_u32(a1, c1, b1);
            let t2 = vbcaxq_u32(b1, a1, c1);
            a1 = t0;
            b1 = t1;
            c1 = t2;

            // rho east
            b0 = vsliq_n_u32(vshrq_n_u32(b0, 32 - 1), b0, 1);
            b1 = vsliq_n_u32(vshrq_n_u32(b1, 32 - 1), b1, 1);
            c0 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c0), rho_east_2));
            c1 = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(c1), rho_east_2));
        }

        // Store results back
        write128(transmute(a0), &mut x[0..16]);
        write128(transmute(b0), &mut x[16..32]);
        write128(transmute(c0), &mut x[32..48]);
        write128(transmute(a1), &mut x[48..64]);
        write128(transmute(b1), &mut x[64..80]);
        write128(transmute(c1), &mut x[80..96]);
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
unsafe fn xoodoo_aarch64_sha3_x4(state: &mut [u8; 192]) {
    use std::arch::aarch64::*;

    unsafe {
        let x: [uint64x2_t; 12] = [
            transmute(read128(&state[0..16])),
            transmute(read128(&state[16..32])),
            transmute(read128(&state[32..48])),
            transmute(read128(&state[48..64])),
            transmute(read128(&state[64..80])),
            transmute(read128(&state[80..96])),
            transmute(read128(&state[96..112])),
            transmute(read128(&state[112..128])),
            transmute(read128(&state[128..144])),
            transmute(read128(&state[144..160])),
            transmute(read128(&state[160..176])),
            transmute(read128(&state[176..192])),
        ];
        let mut x: [uint32x4_t; 12] = [
            transmute(vtrn1q_u64(x[0], x[6])),
            transmute(vtrn1q_u64(x[1], x[7])),
            transmute(vtrn1q_u64(x[2], x[8])),
            transmute(vtrn1q_u64(x[3], x[9])),
            transmute(vtrn1q_u64(x[4], x[10])),
            transmute(vtrn1q_u64(x[5], x[11])),
            transmute(vtrn2q_u64(x[0], x[6])),
            transmute(vtrn2q_u64(x[1], x[7])),
            transmute(vtrn2q_u64(x[2], x[8])),
            transmute(vtrn2q_u64(x[3], x[9])),
            transmute(vtrn2q_u64(x[4], x[10])),
            transmute(vtrn2q_u64(x[5], x[11])),
        ];
        x = [
            vtrn1q_u32(x[0], x[3]),
            vtrn1q_u32(x[1], x[4]),
            vtrn1q_u32(x[2], x[5]),
            vtrn2q_u32(x[0], x[3]),
            vtrn2q_u32(x[1], x[4]),
            vtrn2q_u32(x[2], x[5]),
            vtrn1q_u32(x[6], x[9]),
            vtrn1q_u32(x[7], x[10]),
            vtrn1q_u32(x[8], x[11]),
            vtrn2q_u32(x[6], x[9]),
            vtrn2q_u32(x[7], x[10]),
            vtrn2q_u32(x[8], x[11]),
        ];

        let rho_east_2: uint8x16_t =
            transmute([3u8, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0 = veor3q_u32(x[0], x[4], x[8]);
            let mut p1 = veor3q_u32(x[1], x[5], x[9]);
            let mut p2 = veor3q_u32(x[2], x[6], x[10]);
            let mut p3 = veor3q_u32(x[3], x[7], x[11]);
            let mut e0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 5), p0, 5);
            let mut e1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 5), p1, 5);
            let mut e2 = vsliq_n_u32(vshrq_n_u32(p2, 32 - 5), p2, 5);
            let mut e3 = vsliq_n_u32(vshrq_n_u32(p3, 32 - 5), p3, 5);
            p0 = vsliq_n_u32(vshrq_n_u32(p0, 32 - 14), p0, 14);
            p1 = vsliq_n_u32(vshrq_n_u32(p1, 32 - 14), p1, 14);
            p2 = vsliq_n_u32(vshrq_n_u32(p2, 32 - 14), p2, 14);
            p3 = vsliq_n_u32(vshrq_n_u32(p3, 32 - 14), p3, 14);
            e0 = veorq_u32(e0, p0);
            e1 = veorq_u32(e1, p1);
            e2 = veorq_u32(e2, p2);
            e3 = veorq_u32(e3, p3);
            (e0, e1, e2, e3) = (e1, e2, e3, e0);
            x[0] = veorq_u32(x[0], e0);
            x[4] = veorq_u32(x[4], e0);
            x[8] = veorq_u32(x[8], e0);
            x[1] = veorq_u32(x[1], e1);
            x[5] = veorq_u32(x[5], e1);
            x[9] = veorq_u32(x[9], e1);
            x[2] = veorq_u32(x[2], e2);
            x[6] = veorq_u32(x[6], e2);
            x[10] = veorq_u32(x[10], e2);
            x[3] = veorq_u32(x[3], e3);
            x[7] = veorq_u32(x[7], e3);
            x[11] = veorq_u32(x[11], e3);

            // rho west
            (x[4], x[5], x[6], x[7]) = (x[7], x[4], x[5], x[6]);
            x[8] = vsliq_n_u32(vshrq_n_u32(x[8], 32 - 11), x[8], 11);
            x[9] = vsliq_n_u32(vshrq_n_u32(x[9], 32 - 11), x[9], 11);
            x[10] = vsliq_n_u32(vshrq_n_u32(x[10], 32 - 11), x[10], 11);
            x[11] = vsliq_n_u32(vshrq_n_u32(x[11], 32 - 11), x[11], 11);

            // iota
            let round_key = vdupq_n_u32(round_key);
            x[0] = veorq_u32(x[0], round_key);

            // chi
            let t0 = vbcaxq_u32(x[0], x[4], x[8]);
            let t1 = vbcaxq_u32(x[4], x[8], x[0]);
            let t2 = vbcaxq_u32(x[8], x[0], x[4]);
            x[0] = t0;
            x[4] = t1;
            x[8] = t2;
            let t0 = vbcaxq_u32(x[1], x[5], x[9]);
            let t1 = vbcaxq_u32(x[5], x[9], x[1]);
            let t2 = vbcaxq_u32(x[9], x[1], x[5]);
            x[1] = t0;
            x[5] = t1;
            x[9] = t2;
            let t0 = vbcaxq_u32(x[2], x[6], x[10]);
            let t1 = vbcaxq_u32(x[6], x[10], x[2]);
            let t2 = vbcaxq_u32(x[10], x[2], x[6]);
            x[2] = t0;
            x[6] = t1;
            x[10] = t2;
            let t0 = vbcaxq_u32(x[3], x[7], x[11]);
            let t1 = vbcaxq_u32(x[7], x[11], x[3]);
            let t2 = vbcaxq_u32(x[11], x[3], x[7]);
            x[3] = t0;
            x[7] = t1;
            x[11] = t2;

            // rho east
            x[4] = vsliq_n_u32(vshrq_n_u32(x[4], 32 - 1), x[4], 1);
            x[5] = vsliq_n_u32(vshrq_n_u32(x[5], 32 - 1), x[5], 1);
            x[6] = vsliq_n_u32(vshrq_n_u32(x[6], 32 - 1), x[6], 1);
            x[7] = vsliq_n_u32(vshrq_n_u32(x[7], 32 - 1), x[7], 1);
            (x[8], x[9], x[10], x[11]) = (x[10], x[11], x[8], x[9]);
            x[8] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[8]), rho_east_2));
            x[9] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[9]), rho_east_2));
            x[10] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[10]), rho_east_2));
            x[11] = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(x[11]), rho_east_2));
        }

        let mut x: [uint64x2_t; 12] = [
            transmute(vtrn1q_u32(x[0], x[3])),
            transmute(vtrn1q_u32(x[1], x[4])),
            transmute(vtrn1q_u32(x[2], x[5])),
            transmute(vtrn2q_u32(x[0], x[3])),
            transmute(vtrn2q_u32(x[1], x[4])),
            transmute(vtrn2q_u32(x[2], x[5])),
            transmute(vtrn1q_u32(x[6], x[9])),
            transmute(vtrn1q_u32(x[7], x[10])),
            transmute(vtrn1q_u32(x[8], x[11])),
            transmute(vtrn2q_u32(x[6], x[9])),
            transmute(vtrn2q_u32(x[7], x[10])),
            transmute(vtrn2q_u32(x[8], x[11])),
        ];
        x = [
            vtrn1q_u64(x[0], x[6]),
            vtrn1q_u64(x[1], x[7]),
            vtrn1q_u64(x[2], x[8]),
            vtrn1q_u64(x[3], x[9]),
            vtrn1q_u64(x[4], x[10]),
            vtrn1q_u64(x[5], x[11]),
            vtrn2q_u64(x[0], x[6]),
            vtrn2q_u64(x[1], x[7]),
            vtrn2q_u64(x[2], x[8]),
            vtrn2q_u64(x[3], x[9]),
            vtrn2q_u64(x[4], x[10]),
            vtrn2q_u64(x[5], x[11]),
        ];

        // Store results back
        write128(transmute(x[0]), &mut state[0..16]);
        write128(transmute(x[1]), &mut state[16..32]);
        write128(transmute(x[2]), &mut state[32..48]);
        write128(transmute(x[3]), &mut state[48..64]);
        write128(transmute(x[4]), &mut state[64..80]);
        write128(transmute(x[5]), &mut state[80..96]);
        write128(transmute(x[6]), &mut state[96..112]);
        write128(transmute(x[7]), &mut state[112..128]);
        write128(transmute(x[8]), &mut state[128..144]);
        write128(transmute(x[9]), &mut state[144..160]);
        write128(transmute(x[10]), &mut state[160..176]);
        write128(transmute(x[11]), &mut state[176..192]);
    }
}

#[inline(never)]
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
        (p0, p1, p2, p3) = (p1, p2, p3, p0);
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

#[inline(never)]
fn xoodoo_scalar_x2(state: &mut [u8; 96]) {
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
        read32(&state[48..52]),
        read32(&state[52..56]),
        read32(&state[56..60]),
        read32(&state[60..64]),
        read32(&state[64..68]),
        read32(&state[68..72]),
        read32(&state[72..76]),
        read32(&state[76..80]),
        read32(&state[80..84]),
        read32(&state[84..88]),
        read32(&state[88..92]),
        read32(&state[92..96]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0a = x[0] ^ x[4] ^ x[8];
        let mut p1a = x[1] ^ x[5] ^ x[9];
        let mut p2a = x[2] ^ x[6] ^ x[10];
        let mut p3a = x[3] ^ x[7] ^ x[11];
        let mut p0b = x[12] ^ x[16] ^ x[20];
        let mut p1b = x[13] ^ x[17] ^ x[21];
        let mut p2b = x[14] ^ x[18] ^ x[22];
        let mut p3b = x[15] ^ x[19] ^ x[23];
        p0a = p0a.rotate_left(5) ^ p0a.rotate_left(14);
        p1a = p1a.rotate_left(5) ^ p1a.rotate_left(14);
        p2a = p2a.rotate_left(5) ^ p2a.rotate_left(14);
        p3a = p3a.rotate_left(5) ^ p3a.rotate_left(14);
        p0b = p0b.rotate_left(5) ^ p0b.rotate_left(14);
        p1b = p1b.rotate_left(5) ^ p1b.rotate_left(14);
        p2b = p2b.rotate_left(5) ^ p2b.rotate_left(14);
        p3b = p3b.rotate_left(5) ^ p3b.rotate_left(14);
        (p0a, p1a, p2a, p3a) = (p1a, p2a, p3a, p0a);
        (p0b, p1b, p2b, p3b) = (p1b, p2b, p3b, p0b);
        x[0] ^= p0a;
        x[4] ^= p0a;
        x[8] ^= p0a;
        x[1] ^= p1a;
        x[5] ^= p1a;
        x[9] ^= p1a;
        x[2] ^= p2a;
        x[6] ^= p2a;
        x[10] ^= p2a;
        x[3] ^= p3a;
        x[7] ^= p3a;
        x[11] ^= p3a;
        x[12] ^= p0b;
        x[16] ^= p0b;
        x[20] ^= p0b;
        x[13] ^= p1b;
        x[17] ^= p1b;
        x[21] ^= p1b;
        x[14] ^= p2b;
        x[18] ^= p2b;
        x[22] ^= p2b;
        x[15] ^= p3b;
        x[19] ^= p3b;
        x[23] ^= p3b;

        // rho west
        (x[4], x[5], x[6], x[7]) = (x[7], x[4], x[5], x[6]);
        (x[16], x[17], x[18], x[19]) = (x[19], x[16], x[17], x[18]);
        x[8] = x[8].rotate_left(11);
        x[9] = x[9].rotate_left(11);
        x[10] = x[10].rotate_left(11);
        x[11] = x[11].rotate_left(11);
        x[20] = x[20].rotate_left(11);
        x[21] = x[21].rotate_left(11);
        x[22] = x[22].rotate_left(11);
        x[23] = x[23].rotate_left(11);

        // iota
        x[0] ^= round_key;
        x[12] ^= round_key;

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
        let t0 = x[12] ^ (!x[16] & x[20]);
        let t1 = x[16] ^ (!x[20] & x[12]);
        let t2 = x[20] ^ (!x[12] & x[16]);
        x[12] = t0;
        x[16] = t1;
        x[20] = t2;
        let t0 = x[13] ^ (!x[17] & x[21]);
        let t1 = x[17] ^ (!x[21] & x[13]);
        let t2 = x[21] ^ (!x[13] & x[17]);
        x[13] = t0;
        x[17] = t1;
        x[21] = t2;
        let t0 = x[14] ^ (!x[18] & x[22]);
        let t1 = x[18] ^ (!x[22] & x[14]);
        let t2 = x[22] ^ (!x[14] & x[18]);
        x[14] = t0;
        x[18] = t1;
        x[22] = t2;
        let t0 = x[15] ^ (!x[19] & x[23]);
        let t1 = x[19] ^ (!x[23] & x[15]);
        let t2 = x[23] ^ (!x[15] & x[19]);
        x[15] = t0;
        x[19] = t1;
        x[23] = t2;

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
        x[16] = x[16].rotate_left(1);
        x[17] = x[17].rotate_left(1);
        x[18] = x[18].rotate_left(1);
        x[19] = x[19].rotate_left(1);
        (x[20], x[21], x[22], x[23]) = (x[22], x[23], x[20], x[21]);
        x[20] = x[20].rotate_left(8);
        x[21] = x[21].rotate_left(8);
        x[22] = x[22].rotate_left(8);
        x[23] = x[23].rotate_left(8);
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
    write32(x[12], &mut state[48..52]);
    write32(x[13], &mut state[52..56]);
    write32(x[14], &mut state[56..60]);
    write32(x[15], &mut state[60..64]);
    write32(x[16], &mut state[64..68]);
    write32(x[17], &mut state[68..72]);
    write32(x[18], &mut state[72..76]);
    write32(x[19], &mut state[76..80]);
    write32(x[20], &mut state[80..84]);
    write32(x[21], &mut state[84..88]);
    write32(x[22], &mut state[88..92]);
    write32(x[23], &mut state[92..96]);
}

#[inline(never)]
fn xoodoo_scalar_x4(state: &mut [u8; 192]) {
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
        read32(&state[48..52]),
        read32(&state[52..56]),
        read32(&state[56..60]),
        read32(&state[60..64]),
        read32(&state[64..68]),
        read32(&state[68..72]),
        read32(&state[72..76]),
        read32(&state[76..80]),
        read32(&state[80..84]),
        read32(&state[84..88]),
        read32(&state[88..92]),
        read32(&state[92..96]),
        read32(&state[96..100]),
        read32(&state[100..104]),
        read32(&state[104..108]),
        read32(&state[108..112]),
        read32(&state[112..116]),
        read32(&state[116..120]),
        read32(&state[120..124]),
        read32(&state[124..128]),
        read32(&state[128..132]),
        read32(&state[132..136]),
        read32(&state[136..140]),
        read32(&state[140..144]),
        read32(&state[144..148]),
        read32(&state[148..152]),
        read32(&state[152..156]),
        read32(&state[156..160]),
        read32(&state[160..164]),
        read32(&state[164..168]),
        read32(&state[168..172]),
        read32(&state[172..176]),
        read32(&state[176..180]),
        read32(&state[180..184]),
        read32(&state[184..188]),
        read32(&state[188..192]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0a = x[0] ^ x[4] ^ x[8];
        let mut p1a = x[1] ^ x[5] ^ x[9];
        let mut p2a = x[2] ^ x[6] ^ x[10];
        let mut p3a = x[3] ^ x[7] ^ x[11];
        let mut p0b = x[12] ^ x[16] ^ x[20];
        let mut p1b = x[13] ^ x[17] ^ x[21];
        let mut p2b = x[14] ^ x[18] ^ x[22];
        let mut p3b = x[15] ^ x[19] ^ x[23];
        let mut p0c = x[24] ^ x[28] ^ x[32];
        let mut p1c = x[25] ^ x[29] ^ x[33];
        let mut p2c = x[26] ^ x[30] ^ x[34];
        let mut p3c = x[27] ^ x[31] ^ x[35];
        let mut p0d = x[36] ^ x[40] ^ x[44];
        let mut p1d = x[37] ^ x[41] ^ x[45];
        let mut p2d = x[38] ^ x[42] ^ x[46];
        let mut p3d = x[39] ^ x[43] ^ x[47];
        p0a = p0a.rotate_left(5) ^ p0a.rotate_left(14);
        p1a = p1a.rotate_left(5) ^ p1a.rotate_left(14);
        p2a = p2a.rotate_left(5) ^ p2a.rotate_left(14);
        p3a = p3a.rotate_left(5) ^ p3a.rotate_left(14);
        p0b = p0b.rotate_left(5) ^ p0b.rotate_left(14);
        p1b = p1b.rotate_left(5) ^ p1b.rotate_left(14);
        p2b = p2b.rotate_left(5) ^ p2b.rotate_left(14);
        p3b = p3b.rotate_left(5) ^ p3b.rotate_left(14);
        p0c = p0c.rotate_left(5) ^ p0c.rotate_left(14);
        p1c = p1c.rotate_left(5) ^ p1c.rotate_left(14);
        p2c = p2c.rotate_left(5) ^ p2c.rotate_left(14);
        p3c = p3c.rotate_left(5) ^ p3c.rotate_left(14);
        p0d = p0d.rotate_left(5) ^ p0d.rotate_left(14);
        p1d = p1d.rotate_left(5) ^ p1d.rotate_left(14);
        p2d = p2d.rotate_left(5) ^ p2d.rotate_left(14);
        p3d = p3d.rotate_left(5) ^ p3d.rotate_left(14);
        (p0a, p1a, p2a, p3a) = (p1a, p2a, p3a, p0a);
        (p0b, p1b, p2b, p3b) = (p1b, p2b, p3b, p0b);
        (p0c, p1c, p2c, p3c) = (p1c, p2c, p3c, p0c);
        (p0d, p1d, p2d, p3d) = (p1d, p2d, p3d, p0d);
        x[0] ^= p0a;
        x[4] ^= p0a;
        x[8] ^= p0a;
        x[1] ^= p1a;
        x[5] ^= p1a;
        x[9] ^= p1a;
        x[2] ^= p2a;
        x[6] ^= p2a;
        x[10] ^= p2a;
        x[3] ^= p3a;
        x[7] ^= p3a;
        x[11] ^= p3a;
        x[12] ^= p0b;
        x[16] ^= p0b;
        x[20] ^= p0b;
        x[13] ^= p1b;
        x[17] ^= p1b;
        x[21] ^= p1b;
        x[14] ^= p2b;
        x[18] ^= p2b;
        x[22] ^= p2b;
        x[15] ^= p3b;
        x[19] ^= p3b;
        x[23] ^= p3b;
        x[24] ^= p0c;
        x[28] ^= p0c;
        x[32] ^= p0c;
        x[25] ^= p1c;
        x[29] ^= p1c;
        x[33] ^= p1c;
        x[26] ^= p2c;
        x[30] ^= p2c;
        x[34] ^= p2c;
        x[27] ^= p3c;
        x[31] ^= p3c;
        x[35] ^= p3c;
        x[36] ^= p0d;
        x[40] ^= p0d;
        x[44] ^= p0d;
        x[37] ^= p1d;
        x[41] ^= p1d;
        x[45] ^= p1d;
        x[38] ^= p2d;
        x[42] ^= p2d;
        x[46] ^= p2d;
        x[39] ^= p3d;
        x[43] ^= p3d;
        x[47] ^= p3d;

        // rho west
        (x[4], x[5], x[6], x[7]) = (x[7], x[4], x[5], x[6]);
        (x[16], x[17], x[18], x[19]) = (x[19], x[16], x[17], x[18]);
        (x[28], x[29], x[30], x[31]) = (x[31], x[28], x[29], x[30]);
        (x[40], x[41], x[42], x[43]) = (x[43], x[40], x[41], x[42]);
        x[8] = x[8].rotate_left(11);
        x[9] = x[9].rotate_left(11);
        x[10] = x[10].rotate_left(11);
        x[11] = x[11].rotate_left(11);
        x[20] = x[20].rotate_left(11);
        x[21] = x[21].rotate_left(11);
        x[22] = x[22].rotate_left(11);
        x[23] = x[23].rotate_left(11);
        x[32] = x[32].rotate_left(11);
        x[33] = x[33].rotate_left(11);
        x[34] = x[34].rotate_left(11);
        x[35] = x[35].rotate_left(11);
        x[44] = x[44].rotate_left(11);
        x[45] = x[45].rotate_left(11);
        x[46] = x[46].rotate_left(11);
        x[47] = x[47].rotate_left(11);

        // iota
        x[0] ^= round_key;
        x[12] ^= round_key;
        x[24] ^= round_key;
        x[36] ^= round_key;

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
        let t0 = x[12] ^ (!x[16] & x[20]);
        let t1 = x[16] ^ (!x[20] & x[12]);
        let t2 = x[20] ^ (!x[12] & x[16]);
        x[12] = t0;
        x[16] = t1;
        x[20] = t2;
        let t0 = x[13] ^ (!x[17] & x[21]);
        let t1 = x[17] ^ (!x[21] & x[13]);
        let t2 = x[21] ^ (!x[13] & x[17]);
        x[13] = t0;
        x[17] = t1;
        x[21] = t2;
        let t0 = x[14] ^ (!x[18] & x[22]);
        let t1 = x[18] ^ (!x[22] & x[14]);
        let t2 = x[22] ^ (!x[14] & x[18]);
        x[14] = t0;
        x[18] = t1;
        x[22] = t2;
        let t0 = x[15] ^ (!x[19] & x[23]);
        let t1 = x[19] ^ (!x[23] & x[15]);
        let t2 = x[23] ^ (!x[15] & x[19]);
        x[15] = t0;
        x[19] = t1;
        x[23] = t2;
        let t0 = x[24] ^ (!x[28] & x[32]);
        let t1 = x[28] ^ (!x[32] & x[24]);
        let t2 = x[32] ^ (!x[24] & x[28]);
        x[24] = t0;
        x[28] = t1;
        x[32] = t2;
        let t0 = x[25] ^ (!x[29] & x[33]);
        let t1 = x[29] ^ (!x[33] & x[25]);
        let t2 = x[33] ^ (!x[25] & x[29]);
        x[25] = t0;
        x[29] = t1;
        x[33] = t2;
        let t0 = x[26] ^ (!x[30] & x[34]);
        let t1 = x[30] ^ (!x[34] & x[26]);
        let t2 = x[34] ^ (!x[26] & x[30]);
        x[26] = t0;
        x[30] = t1;
        x[34] = t2;
        let t0 = x[27] ^ (!x[31] & x[35]);
        let t1 = x[31] ^ (!x[35] & x[27]);
        let t2 = x[35] ^ (!x[27] & x[31]);
        x[27] = t0;
        x[31] = t1;
        x[35] = t2;
        let t0 = x[36] ^ (!x[40] & x[44]);
        let t1 = x[40] ^ (!x[44] & x[36]);
        let t2 = x[44] ^ (!x[36] & x[40]);
        x[36] = t0;
        x[40] = t1;
        x[44] = t2;
        let t0 = x[37] ^ (!x[41] & x[45]);
        let t1 = x[41] ^ (!x[45] & x[37]);
        let t2 = x[45] ^ (!x[37] & x[41]);
        x[37] = t0;
        x[41] = t1;
        x[45] = t2;
        let t0 = x[38] ^ (!x[42] & x[46]);
        let t1 = x[42] ^ (!x[46] & x[38]);
        let t2 = x[46] ^ (!x[38] & x[42]);
        x[38] = t0;
        x[42] = t1;
        x[46] = t2;
        let t0 = x[39] ^ (!x[43] & x[47]);
        let t1 = x[43] ^ (!x[47] & x[39]);
        let t2 = x[47] ^ (!x[39] & x[43]);
        x[39] = t0;
        x[43] = t1;
        x[47] = t2;

        // rho east
        x[4] = x[4].rotate_left(1);
        x[5] = x[5].rotate_left(1);
        x[6] = x[6].rotate_left(1);
        x[7] = x[7].rotate_left(1);
        x[16] = x[16].rotate_left(1);
        x[17] = x[17].rotate_left(1);
        x[18] = x[18].rotate_left(1);
        x[19] = x[19].rotate_left(1);
        x[28] = x[28].rotate_left(1);
        x[29] = x[29].rotate_left(1);
        x[30] = x[30].rotate_left(1);
        x[31] = x[31].rotate_left(1);
        x[40] = x[40].rotate_left(1);
        x[41] = x[41].rotate_left(1);
        x[42] = x[42].rotate_left(1);
        x[43] = x[43].rotate_left(1);
        (x[8], x[9], x[10], x[11]) = (x[10], x[11], x[8], x[9]);
        (x[20], x[21], x[22], x[23]) = (x[22], x[23], x[20], x[21]);
        (x[32], x[33], x[34], x[35]) = (x[34], x[35], x[32], x[33]);
        (x[44], x[45], x[46], x[47]) = (x[46], x[47], x[44], x[45]);
        x[8] = x[8].rotate_left(8);
        x[9] = x[9].rotate_left(8);
        x[10] = x[10].rotate_left(8);
        x[11] = x[11].rotate_left(8);
        x[20] = x[20].rotate_left(8);
        x[21] = x[21].rotate_left(8);
        x[22] = x[22].rotate_left(8);
        x[23] = x[23].rotate_left(8);
        x[32] = x[32].rotate_left(8);
        x[33] = x[33].rotate_left(8);
        x[34] = x[34].rotate_left(8);
        x[35] = x[35].rotate_left(8);
        x[44] = x[44].rotate_left(8);
        x[45] = x[45].rotate_left(8);
        x[46] = x[46].rotate_left(8);
        x[47] = x[47].rotate_left(8);
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
    write32(x[12], &mut state[48..52]);
    write32(x[13], &mut state[52..56]);
    write32(x[14], &mut state[56..60]);
    write32(x[15], &mut state[60..64]);
    write32(x[16], &mut state[64..68]);
    write32(x[17], &mut state[68..72]);
    write32(x[18], &mut state[72..76]);
    write32(x[19], &mut state[76..80]);
    write32(x[20], &mut state[80..84]);
    write32(x[21], &mut state[84..88]);
    write32(x[22], &mut state[88..92]);
    write32(x[23], &mut state[92..96]);
    write32(x[24], &mut state[96..100]);
    write32(x[25], &mut state[100..104]);
    write32(x[26], &mut state[104..108]);
    write32(x[27], &mut state[108..112]);
    write32(x[28], &mut state[112..116]);
    write32(x[29], &mut state[116..120]);
    write32(x[30], &mut state[120..124]);
    write32(x[31], &mut state[124..128]);
    write32(x[32], &mut state[128..132]);
    write32(x[33], &mut state[132..136]);
    write32(x[34], &mut state[136..140]);
    write32(x[35], &mut state[140..144]);
    write32(x[36], &mut state[144..148]);
    write32(x[37], &mut state[148..152]);
    write32(x[38], &mut state[152..156]);
    write32(x[39], &mut state[156..160]);
    write32(x[40], &mut state[160..164]);
    write32(x[41], &mut state[164..168]);
    write32(x[42], &mut state[168..172]);
    write32(x[43], &mut state[172..176]);
    write32(x[44], &mut state[176..180]);
    write32(x[45], &mut state[180..184]);
    write32(x[46], &mut state[184..188]);
    write32(x[47], &mut state[188..192]);
}

/// A variant of Xoodoo that is based on 64-bit words.
#[inline(never)]
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

#[inline(never)]
fn xoodoo64_scalar_x2(state: &mut [u8; 96]) {
    let mut x = [
        read64(&state[0..8]),
        read64(&state[8..16]),
        read64(&state[16..24]),
        read64(&state[24..32]),
        read64(&state[32..40]),
        read64(&state[40..48]),
        read64(&state[48..56]),
        read64(&state[56..64]),
        read64(&state[64..72]),
        read64(&state[72..80]),
        read64(&state[80..88]),
        read64(&state[88..96]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0a = x[0] ^ x[2] ^ x[4];
        let mut p1a = x[1] ^ x[3] ^ x[5];
        let mut p0b = x[6] ^ x[8] ^ x[10];
        let mut p1b = x[7] ^ x[9] ^ x[11];
        p0a = p0a.rotate_left(10) ^ p0a.rotate_left(29);
        p1a = p1a.rotate_left(10) ^ p1a.rotate_left(29);
        p0b = p0b.rotate_left(10) ^ p0b.rotate_left(29);
        p1b = p1b.rotate_left(10) ^ p1b.rotate_left(29);
        (p0a, p1a) = (p1a, p0a);
        (p0b, p1b) = (p1b, p0b);

        x[0] ^= p0a;
        x[2] ^= p0a;
        x[4] ^= p0a;
        x[1] ^= p1a;
        x[3] ^= p1a;
        x[5] ^= p1a;
        x[6] ^= p0b;
        x[8] ^= p0b;
        x[10] ^= p0b;
        x[7] ^= p1b;
        x[9] ^= p1b;
        x[11] ^= p1b;

        // rho west
        x.swap(2, 3);
        x.swap(8, 9);
        x[4] = x[4].rotate_left(23);
        x[5] = x[5].rotate_left(23);
        x[10] = x[10].rotate_left(23);
        x[11] = x[11].rotate_left(23);

        // iota
        x[0] ^= round_key as u64;
        x[6] ^= round_key as u64;
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
        let t0 = x[6] ^ (!x[8] & x[10]);
        let t1 = x[8] ^ (!x[10] & x[6]);
        let t2 = x[10] ^ (!x[6] & x[8]);
        x[6] = t0;
        x[8] = t1;
        x[10] = t2;
        let t0 = x[7] ^ (!x[9] & x[11]);
        let t1 = x[9] ^ (!x[11] & x[7]);
        let t2 = x[11] ^ (!x[7] & x[9]);
        x[7] = t0;
        x[9] = t1;
        x[11] = t2;

        // rho east
        x[2] = x[2].rotate_left(1);
        x[3] = x[3].rotate_left(1);
        x[8] = x[8].rotate_left(1);
        x[9] = x[9].rotate_left(1);
        x.swap(4, 5);
        x.swap(10, 11);
        x[4] = x[4].rotate_left(16);
        x[5] = x[5].rotate_left(16);
        x[10] = x[10].rotate_left(16);
        x[11] = x[11].rotate_left(16);
    }

    write64(x[0], &mut state[0..8]);
    write64(x[1], &mut state[8..16]);
    write64(x[2], &mut state[16..24]);
    write64(x[3], &mut state[24..32]);
    write64(x[4], &mut state[32..40]);
    write64(x[5], &mut state[40..48]);
    write64(x[6], &mut state[48..56]);
    write64(x[7], &mut state[56..64]);
    write64(x[8], &mut state[64..72]);
    write64(x[9], &mut state[72..80]);
    write64(x[10], &mut state[80..88]);
    write64(x[11], &mut state[88..96]);
}

#[inline(never)]
fn xoodoo64_scalar_x4(state: &mut [u8; 192]) {
    let mut x = [
        read64(&state[0..8]),
        read64(&state[8..16]),
        read64(&state[16..24]),
        read64(&state[24..32]),
        read64(&state[32..40]),
        read64(&state[40..48]),
        read64(&state[48..56]),
        read64(&state[56..64]),
        read64(&state[64..72]),
        read64(&state[72..80]),
        read64(&state[80..88]),
        read64(&state[88..96]),
        read64(&state[96..104]),
        read64(&state[104..112]),
        read64(&state[112..120]),
        read64(&state[120..128]),
        read64(&state[128..136]),
        read64(&state[136..144]),
        read64(&state[144..152]),
        read64(&state[152..160]),
        read64(&state[160..168]),
        read64(&state[168..176]),
        read64(&state[176..184]),
        read64(&state[184..192]),
    ];

    for &round_key in &ROUND_KEYS {
        // theta
        let mut p0a = x[0] ^ x[2] ^ x[4];
        let mut p1a = x[1] ^ x[3] ^ x[5];
        let mut p0b = x[6] ^ x[8] ^ x[10];
        let mut p1b = x[7] ^ x[9] ^ x[11];
        let mut p0c = x[12] ^ x[14] ^ x[16];
        let mut p1c = x[13] ^ x[15] ^ x[17];
        let mut p0d = x[18] ^ x[20] ^ x[22];
        let mut p1d = x[19] ^ x[21] ^ x[23];
        p0a = p0a.rotate_left(10) ^ p0a.rotate_left(29);
        p1a = p1a.rotate_left(10) ^ p1a.rotate_left(29);
        p0b = p0b.rotate_left(10) ^ p0b.rotate_left(29);
        p1b = p1b.rotate_left(10) ^ p1b.rotate_left(29);
        p0c = p0c.rotate_left(10) ^ p0c.rotate_left(29);
        p1c = p1c.rotate_left(10) ^ p1c.rotate_left(29);
        p0d = p0d.rotate_left(10) ^ p0d.rotate_left(29);
        p1d = p1d.rotate_left(10) ^ p1d.rotate_left(29);
        (p0a, p1a) = (p1a, p0a);
        (p0b, p1b) = (p1b, p0b);
        (p0c, p1c) = (p1c, p0c);
        (p0d, p1d) = (p1d, p0d);

        x[0] ^= p0a;
        x[2] ^= p0a;
        x[4] ^= p0a;
        x[1] ^= p1a;
        x[3] ^= p1a;
        x[5] ^= p1a;
        x[6] ^= p0b;
        x[8] ^= p0b;
        x[10] ^= p0b;
        x[7] ^= p1b;
        x[9] ^= p1b;
        x[11] ^= p1b;
        x[12] ^= p0c;
        x[14] ^= p0c;
        x[16] ^= p0c;
        x[13] ^= p1c;
        x[15] ^= p1c;
        x[17] ^= p1c;
        x[18] ^= p0d;
        x[20] ^= p0d;
        x[22] ^= p0d;
        x[19] ^= p1d;
        x[21] ^= p1d;
        x[23] ^= p1d;

        // rho west
        x.swap(2, 3);
        x.swap(8, 9);
        x.swap(14, 15);
        x.swap(20, 21);
        x[4] = x[4].rotate_left(23);
        x[5] = x[5].rotate_left(23);
        x[10] = x[10].rotate_left(23);
        x[11] = x[11].rotate_left(23);
        x[16] = x[16].rotate_left(23);
        x[17] = x[17].rotate_left(23);
        x[22] = x[22].rotate_left(23);
        x[23] = x[23].rotate_left(23);

        // iota
        x[0] ^= round_key as u64;
        x[6] ^= round_key as u64;
        x[12] ^= round_key as u64;
        x[18] ^= round_key as u64;

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
        let t0 = x[6] ^ (!x[8] & x[10]);
        let t1 = x[8] ^ (!x[10] & x[6]);
        let t2 = x[10] ^ (!x[6] & x[8]);
        x[6] = t0;
        x[8] = t1;
        x[10] = t2;
        let t0 = x[7] ^ (!x[9] & x[11]);
        let t1 = x[9] ^ (!x[11] & x[7]);
        let t2 = x[11] ^ (!x[7] & x[9]);
        x[7] = t0;
        x[9] = t1;
        x[11] = t2;
        let t0 = x[12] ^ (!x[14] & x[16]);
        let t1 = x[14] ^ (!x[16] & x[12]);
        let t2 = x[16] ^ (!x[12] & x[14]);
        x[12] = t0;
        x[14] = t1;
        x[16] = t2;
        let t0 = x[13] ^ (!x[15] & x[17]);
        let t1 = x[15] ^ (!x[17] & x[13]);
        let t2 = x[17] ^ (!x[13] & x[15]);
        x[13] = t0;
        x[15] = t1;
        x[17] = t2;
        let t0 = x[18] ^ (!x[20] & x[22]);
        let t1 = x[20] ^ (!x[22] & x[18]);
        let t2 = x[22] ^ (!x[18] & x[20]);
        x[18] = t0;
        x[20] = t1;
        x[22] = t2;
        let t0 = x[19] ^ (!x[21] & x[23]);
        let t1 = x[21] ^ (!x[23] & x[19]);
        let t2 = x[23] ^ (!x[19] & x[21]);
        x[19] = t0;
        x[21] = t1;
        x[23] = t2;

        // rho east
        x[2] = x[2].rotate_left(1);
        x[3] = x[3].rotate_left(1);
        x[8] = x[8].rotate_left(1);
        x[9] = x[9].rotate_left(1);
        x[14] = x[14].rotate_left(1);
        x[15] = x[15].rotate_left(1);
        x[20] = x[20].rotate_left(1);
        x[21] = x[21].rotate_left(1);
        x.swap(4, 5);
        x.swap(10, 11);
        x.swap(16, 17);
        x.swap(22, 23);
        x[4] = x[4].rotate_left(16);
        x[5] = x[5].rotate_left(16);
        x[10] = x[10].rotate_left(16);
        x[11] = x[11].rotate_left(16);
        x[16] = x[16].rotate_left(16);
        x[17] = x[17].rotate_left(16);
        x[22] = x[22].rotate_left(16);
        x[23] = x[23].rotate_left(16);
    }

    write64(x[0], &mut state[0..8]);
    write64(x[1], &mut state[8..16]);
    write64(x[2], &mut state[16..24]);
    write64(x[3], &mut state[24..32]);
    write64(x[4], &mut state[32..40]);
    write64(x[5], &mut state[40..48]);
    write64(x[6], &mut state[48..56]);
    write64(x[7], &mut state[56..64]);
    write64(x[8], &mut state[64..72]);
    write64(x[9], &mut state[72..80]);
    write64(x[10], &mut state[80..88]);
    write64(x[11], &mut state[88..96]);
    write64(x[12], &mut state[96..104]);
    write64(x[13], &mut state[104..112]);
    write64(x[14], &mut state[112..120]);
    write64(x[15], &mut state[120..128]);
    write64(x[16], &mut state[128..136]);
    write64(x[17], &mut state[136..144]);
    write64(x[18], &mut state[144..152]);
    write64(x[19], &mut state[152..160]);
    write64(x[20], &mut state[160..168]);
    write64(x[21], &mut state[168..176]);
    write64(x[22], &mut state[176..184]);
    write64(x[23], &mut state[184..192]);
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
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

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
unsafe fn xoodoo64_aarch64_sha3_x2(state: &mut [u8; 96]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut x: [uint64x2_t; 6] = [
            transmute(read128(&state[0..16])),
            transmute(read128(&state[16..32])),
            transmute(read128(&state[32..48])),
            transmute(read128(&state[48..64])),
            transmute(read128(&state[64..80])),
            transmute(read128(&state[80..96])),
        ];
        // Transpose from grouped-by-block to grouped-by-word.
        x = [
            vtrn1q_u64(x[0], x[3]),
            vtrn1q_u64(x[1], x[4]),
            vtrn1q_u64(x[2], x[5]),
            vtrn2q_u64(x[0], x[3]),
            vtrn2q_u64(x[1], x[4]),
            vtrn2q_u64(x[2], x[5]),
        ];

        let zero: uint64x2_t = transmute([0u8; 16]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0 = veor3q_u64(x[0], x[2], x[4]);
            let mut p1 = veor3q_u64(x[1], x[3], x[5]);
            p0 = vxarq_u64::<10>(p0, vxarq_u64::<19>(p0, zero));
            p1 = vxarq_u64::<10>(p1, vxarq_u64::<19>(p1, zero));
            (p0, p1) = (p1, p0);

            x[0] = veorq_u64(x[0], p0);
            x[2] = veorq_u64(x[2], p0);
            x[4] = veorq_u64(x[4], p0);
            x[1] = veorq_u64(x[1], p1);
            x[3] = veorq_u64(x[3], p1);
            x[5] = veorq_u64(x[5], p1);

            // rho west
            x.swap(2, 3);
            x[4] = vxarq_u64::<23>(x[4], zero);
            x[5] = vxarq_u64::<23>(x[5], zero);

            // iota
            x[0] = veorq_u64(x[0], vdupq_n_u64(round_key as u64));

            // chi
            let t0 = vbcaxq_u64(x[4], x[2], x[0]);
            let t1 = vbcaxq_u64(x[0], x[4], x[2]);
            let t2 = vbcaxq_u64(x[2], x[0], x[4]);
            x[0] = t0;
            x[2] = t1;
            x[4] = t2;
            let t0 = vbcaxq_u64(x[5], x[3], x[1]);
            let t1 = vbcaxq_u64(x[1], x[5], x[3]);
            let t2 = vbcaxq_u64(x[3], x[1], x[5]);
            x[1] = t0;
            x[3] = t1;
            x[5] = t2;

            // rho east
            x[2] = vxarq_u64::<1>(x[2], zero);
            x[3] = vxarq_u64::<1>(x[3], zero);
            x.swap(4, 5);
            x[4] = vxarq_u64::<16>(x[4], zero);
            x[5] = vxarq_u64::<16>(x[5], zero);
        }

        // Transpose back to grouped-by-block
        x = [
            vtrn1q_u64(x[0], x[3]),
            vtrn1q_u64(x[1], x[4]),
            vtrn1q_u64(x[2], x[5]),
            vtrn2q_u64(x[0], x[3]),
            vtrn2q_u64(x[1], x[4]),
            vtrn2q_u64(x[2], x[5]),
        ];

        // Store results back
        write128(transmute(x[0]), &mut state[0..16]);
        write128(transmute(x[1]), &mut state[16..32]);
        write128(transmute(x[2]), &mut state[32..48]);
        write128(transmute(x[3]), &mut state[48..64]);
        write128(transmute(x[4]), &mut state[64..80]);
        write128(transmute(x[5]), &mut state[80..96]);
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "sha3")]
#[inline(never)]
unsafe fn xoodoo64_aarch64_sha3_x4(state: &mut [u8; 192]) {
    use std::arch::aarch64::*;

    unsafe {
        let mut x: [uint64x2_t; 12] = [
            transmute(read128(&state[0..16])),
            transmute(read128(&state[16..32])),
            transmute(read128(&state[32..48])),
            transmute(read128(&state[48..64])),
            transmute(read128(&state[64..80])),
            transmute(read128(&state[80..96])),
            transmute(read128(&state[96..112])),
            transmute(read128(&state[112..128])),
            transmute(read128(&state[128..144])),
            transmute(read128(&state[144..160])),
            transmute(read128(&state[160..176])),
            transmute(read128(&state[176..192])),
        ];
        // Transpose from grouped-by-block to grouped-by-word.
        x = [
            vtrn1q_u64(x[0], x[3]),
            vtrn1q_u64(x[1], x[4]),
            vtrn1q_u64(x[2], x[5]),
            vtrn2q_u64(x[0], x[3]),
            vtrn2q_u64(x[1], x[4]),
            vtrn2q_u64(x[2], x[5]),
            vtrn1q_u64(x[6], x[9]),
            vtrn1q_u64(x[7], x[10]),
            vtrn1q_u64(x[8], x[11]),
            vtrn2q_u64(x[6], x[9]),
            vtrn2q_u64(x[7], x[10]),
            vtrn2q_u64(x[8], x[11]),
        ];

        let zero: uint64x2_t = transmute([0u8; 16]);

        for &round_key in &ROUND_KEYS {
            // theta
            let mut p0a = veor3q_u64(x[0], x[2], x[4]);
            let mut p1a = veor3q_u64(x[1], x[3], x[5]);
            let mut p0b = veor3q_u64(x[6], x[8], x[10]);
            let mut p1b = veor3q_u64(x[7], x[9], x[11]);
            p0a = vxarq_u64::<10>(p0a, vxarq_u64::<19>(p0a, zero));
            p1a = vxarq_u64::<10>(p1a, vxarq_u64::<19>(p1a, zero));
            p0b = vxarq_u64::<10>(p0b, vxarq_u64::<19>(p0b, zero));
            p1b = vxarq_u64::<10>(p1b, vxarq_u64::<19>(p1b, zero));
            (p0a, p1a) = (p1a, p0a);
            (p0b, p1b) = (p1b, p0b);

            x[0] = veorq_u64(x[0], p0a);
            x[2] = veorq_u64(x[2], p0a);
            x[4] = veorq_u64(x[4], p0a);
            x[1] = veorq_u64(x[1], p1a);
            x[3] = veorq_u64(x[3], p1a);
            x[5] = veorq_u64(x[5], p1a);
            x[6] = veorq_u64(x[6], p0b);
            x[8] = veorq_u64(x[8], p0b);
            x[10] = veorq_u64(x[10], p0b);
            x[7] = veorq_u64(x[7], p1b);
            x[9] = veorq_u64(x[9], p1b);
            x[11] = veorq_u64(x[11], p1b);

            // rho west
            x.swap(2, 3);
            x.swap(8, 9);
            x[4] = vxarq_u64::<23>(x[4], zero);
            x[5] = vxarq_u64::<23>(x[5], zero);
            x[10] = vxarq_u64::<23>(x[10], zero);
            x[11] = vxarq_u64::<23>(x[11], zero);

            // iota
            x[0] = veorq_u64(x[0], vdupq_n_u64(round_key as u64));
            x[6] = veorq_u64(x[6], vdupq_n_u64(round_key as u64));

            // chi
            let t0 = vbcaxq_u64(x[4], x[2], x[0]);
            let t1 = vbcaxq_u64(x[0], x[4], x[2]);
            let t2 = vbcaxq_u64(x[2], x[0], x[4]);
            x[0] = t0;
            x[2] = t1;
            x[4] = t2;
            let t0 = vbcaxq_u64(x[5], x[3], x[1]);
            let t1 = vbcaxq_u64(x[1], x[5], x[3]);
            let t2 = vbcaxq_u64(x[3], x[1], x[5]);
            x[1] = t0;
            x[3] = t1;
            x[5] = t2;
            let t0 = vbcaxq_u64(x[10], x[8], x[6]);
            let t1 = vbcaxq_u64(x[6], x[10], x[8]);
            let t2 = vbcaxq_u64(x[8], x[6], x[10]);
            x[6] = t0;
            x[8] = t1;
            x[10] = t2;
            let t0 = vbcaxq_u64(x[11], x[9], x[7]);
            let t1 = vbcaxq_u64(x[7], x[11], x[9]);
            let t2 = vbcaxq_u64(x[9], x[7], x[11]);
            x[7] = t0;
            x[9] = t1;
            x[11] = t2;

            // rho east
            x[2] = vxarq_u64::<1>(x[2], zero);
            x[3] = vxarq_u64::<1>(x[3], zero);
            x[8] = vxarq_u64::<1>(x[8], zero);
            x[9] = vxarq_u64::<1>(x[9], zero);
            x.swap(4, 5);
            x.swap(10, 11);
            x[4] = vxarq_u64::<16>(x[4], zero);
            x[5] = vxarq_u64::<16>(x[5], zero);
            x[10] = vxarq_u64::<16>(x[10], zero);
            x[11] = vxarq_u64::<16>(x[11], zero);
        }

        // Transpose back to grouped-by-block
        x = [
            vtrn1q_u64(x[0], x[3]),
            vtrn1q_u64(x[1], x[4]),
            vtrn1q_u64(x[2], x[5]),
            vtrn2q_u64(x[0], x[3]),
            vtrn2q_u64(x[1], x[4]),
            vtrn2q_u64(x[2], x[5]),
            vtrn1q_u64(x[6], x[9]),
            vtrn1q_u64(x[7], x[10]),
            vtrn1q_u64(x[8], x[11]),
            vtrn2q_u64(x[6], x[9]),
            vtrn2q_u64(x[7], x[10]),
            vtrn2q_u64(x[8], x[11]),
        ];

        // Store results back
        write128(transmute(x[0]), &mut state[0..16]);
        write128(transmute(x[1]), &mut state[16..32]);
        write128(transmute(x[2]), &mut state[32..48]);
        write128(transmute(x[3]), &mut state[48..64]);
        write128(transmute(x[4]), &mut state[64..80]);
        write128(transmute(x[5]), &mut state[80..96]);
        write128(transmute(x[6]), &mut state[96..112]);
        write128(transmute(x[7]), &mut state[112..128]);
        write128(transmute(x[8]), &mut state[128..144]);
        write128(transmute(x[9]), &mut state[144..160]);
        write128(transmute(x[10]), &mut state[160..176]);
        write128(transmute(x[11]), &mut state[176..192]);
    }
}

#[inline(never)]
fn benchmark<const N: usize>(name: &str, parallelism: usize, f: impl Fn(&mut [u8; N])) {
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
    let throughput = elapsed.as_nanos() as f64 / (parallelism * ITERS) as f64;
    println!(
        "{:<30} {:>16.1} {:4}x {:16.1}",
        name, latency, parallelism, throughput
    );
}

fn main() {
    println!(
        "{:<30} {:>16}  {:>4} {:>16}",
        "name", "latency (ns)", "par", "throughput (ns)"
    );
    benchmark("xoodoo_scalar", 1, xoodoo_scalar);
    benchmark("xoodoo_scalar_x2", 2, xoodoo_scalar_x2);
    benchmark("xoodoo_scalar_x4", 4, xoodoo_scalar_x4);
    benchmark("xoodoo64_scalar", 1, xoodoo64_scalar);
    benchmark("xoodoo64_scalar_x2", 2, xoodoo64_scalar_x2);
    benchmark("xoodoo64_scalar_x4", 4, xoodoo64_scalar_x4);
    benchmark("xoodoo_aarch64", 1, xoodoo_aarch64);
    benchmark("xoodoo_aarch64_x2", 2, xoodoo_aarch64_x2);
    benchmark("xoodoo_aarch64_x4", 4, xoodoo_aarch64_x4);
    benchmark("xoodoo_aarch64_sha3", 1, |x| unsafe {
        xoodoo_aarch64_sha3(x)
    });
    benchmark("xoodoo_aarch64_sha3_x2", 2, |x| unsafe {
        xoodoo_aarch64_sha3_x2(x)
    });
    benchmark("xoodoo_aarch64_sha3_x4", 4, |x| unsafe {
        xoodoo_aarch64_sha3_x4(x)
    });
    benchmark("xoodoo64_aarch64_sha3", 1, |x| unsafe {
        xoodoo64_aarch64_sha3(x)
    });
    benchmark("xoodoo64_aarch64_sha3_x2", 2, |x| unsafe {
        xoodoo64_aarch64_sha3_x2(x)
    });
    benchmark("xoodoo64_aarch64_sha3_x4", 4, |x| unsafe {
        xoodoo64_aarch64_sha3_x4(x)
    });
}
