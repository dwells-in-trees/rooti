pub(crate) fn hash_u64(seed: u64, tick: u64, node_id: u32, purpose: u32) -> u64 {
    let mut x = seed;
    x = x.wrapping_add(tick.wrapping_mul(0x9E3779B97F4A7C15));
    x = x.wrapping_add(((node_id as u64) << 32) | (purpose as u64));
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

pub(crate) const PURPOSE_GROWTH: u32 = 0;
pub(crate) const _PURPOSE_ELEVATION_NOISE: u32 = 1;
pub(crate) const _PURPOSE_AZIMUTH_NOISE: u32 = 2;
pub(crate) const PURPOSE_FIRST_BRANCH: u32 = 3;
pub(crate) const PURPOSE_CONTINUATION_NOISE: u32 = 4;
pub(crate) const PURPOSE_BRANCH_NOISE: u32 = 5;
pub(crate) const PURPOSE_LEAF_NOISE: u32 = 6;
pub(crate) const PURPOSE_BRANCH_THRESHOLD: u32 = 7;

pub(crate) fn grows(seed: u64, tick: u64, node_id: u32, probability: f32) -> bool {
    let h = hash_u64(seed, tick, node_id, PURPOSE_GROWTH);
    // Convert to [0, 1) f64 by taking top 53 bits (f64 mantissa width)
    let unit = (h >> 11) as f64 / (1u64 << 53) as f64;
    unit < probability as f64
}

pub(crate) fn noise_f32(seed: u64, tick: u64, node_id: u32, purpose: u32, range: f32) -> f32 {
    let h = hash_u64(seed, tick, node_id, purpose);
    // Convert to [-1, 1) f32
    let unit = (h >> 40) as f32 / (1u32 << 24) as f32; // [0, 1)
    (unit * 2.0 - 1.0) * range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(hash_u64(42, 100, 7, PURPOSE_GROWTH), hash_u64(42, 100, 7, PURPOSE_GROWTH));
    }

    #[test]
    fn hash_distinguishes_purposes() {
        let a = hash_u64(42, 100, 7, PURPOSE_GROWTH);
        let b = hash_u64(42, 100, 7, PURPOSE_FIRST_BRANCH);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_distinguishes_ticks() {
        assert_ne!(hash_u64(42, 100, 7, 0), hash_u64(42, 101, 7, 0));
    }

    #[test]
    fn hash_distinguishes_nodes() {
        assert_ne!(hash_u64(42, 100, 7, 0), hash_u64(42, 100, 8, 0));
    }
}