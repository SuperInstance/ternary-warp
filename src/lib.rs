#![forbid(unsafe_code)]

pub fn clamp(values: &[i8], min: i8, max: i8) -> Vec<i8> {
    values.iter().map(|&v| v.clamp(min, max)).collect()
}

pub fn quantize(values: &[f64], thresholds: (f64, f64)) -> Vec<i8> {
    values.iter().map(|&v| {
        if v < thresholds.0 { -1 }
        else if v > thresholds.1 { 1 }
        else { 0 }
    }).collect()
}

pub fn fold(values: &[i8], f: fn(i8, i8) -> i8) -> Vec<i8> {
    values.chunks(2).map(|chunk| {
        if chunk.len() == 2 {
            f(chunk[0], chunk[1])
        } else {
            chunk[0]
        }
    }).collect()
}

pub fn warp(values: &[i8], map: fn(i8) -> i8) -> Vec<i8> {
    values.iter().map(|&v| map(v)).collect()
}

pub fn smooth(values: &[i8], radius: usize) -> Vec<i8> {
    if radius == 0 || values.is_empty() {
        return values.to_vec();
    }
    values.iter().enumerate().map(|(i, _)| {
        let lo = i.saturating_sub(radius);
        let hi = (i + radius + 1).min(values.len());
        let window = &values[lo..hi];
        let counts = [
            window.iter().filter(|&&v| v == -1).count(),
            window.iter().filter(|&&v| v == 0).count(),
            window.iter().filter(|&&v| v == 1).count(),
        ];
        if counts[0] >= counts[1] && counts[0] >= counts[2] { -1 }
        else if counts[2] >= counts[0] && counts[2] >= counts[1] { 1 }
        else { 0 }
    }).collect()
}

pub fn differentiate(values: &[i8]) -> Vec<i8> {
    if values.is_empty() {
        return vec![];
    }
    std::iter::once(0)
        .chain(values.windows(2).map(|w| (w[1] - w[0]).clamp(-1, 1)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_no_change() {
        assert_eq!(clamp(&[-1, 0, 1], -1, 1), vec![-1, 0, 1]);
    }

    #[test]
    fn test_clamp_trims() {
        assert_eq!(clamp(&[5, -3, 0], -1, 1), vec![1, -1, 0]);
    }

    #[test]
    fn test_quantize_negative() {
        assert_eq!(quantize(&[-0.8], (-0.3, 0.3)), vec![-1]);
    }

    #[test]
    fn test_quantize_zero() {
        assert_eq!(quantize(&[0.0], (-0.3, 0.3)), vec![0]);
    }

    #[test]
    fn test_quantize_positive() {
        assert_eq!(quantize(&[0.8], (-0.3, 0.3)), vec![1]);
    }

    #[test]
    fn test_fold_pairs() {
        let r = fold(&[1, -1, 0, 1], |a, b| a + b);
        assert_eq!(r, vec![0, 1]);
    }

    #[test]
    fn test_fold_odd() {
        let r = fold(&[1, -1, 1], |a, b| a + b);
        assert_eq!(r, vec![0, 1]);
    }

    #[test]
    fn test_warp_identity() {
        assert_eq!(warp(&[-1, 0, 1], |v| v), vec![-1, 0, 1]);
    }

    #[test]
    fn test_warp_negate() {
        assert_eq!(warp(&[-1, 0, 1], |v| -v), vec![1, 0, -1]);
    }

    #[test]
    fn test_smooth_flat() {
        assert_eq!(smooth(&[0, 0, 0], 1), vec![0, 0, 0]);
    }

    #[test]
    fn test_smooth_removes_noise() {
        // [1, 1, -1, 1, 1] with radius 1 → majority at each position is 1
        let r = smooth(&[1, 1, -1, 1, 1], 1);
        assert_eq!(r, vec![1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_differentiate_flat() {
        assert_eq!(differentiate(&[0, 0, 0]), vec![0, 0, 0]);
    }

    #[test]
    fn test_differentiate_rise() {
        assert_eq!(differentiate(&[-1, 0, 1]), vec![0, 1, 1]);
    }

    #[test]
    fn test_differentiate_empty() {
        assert_eq!(differentiate(&[]), vec![]);
    }

    #[test]
    fn test_smooth_radius_zero() {
        assert_eq!(smooth(&[1, -1, 0], 0), vec![1, -1, 0]);
    }
}
