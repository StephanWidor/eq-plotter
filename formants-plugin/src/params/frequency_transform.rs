use super::*;
use std::ops::RangeInclusive;

/// For transforming/distorting coordinates (x,y) in [0;1]^2 into formant frequencies
/// (see doc folder for explanation)
pub struct FrequencyTransform {
    pub bounds_matrix: nalgebra::Matrix2x4<f32>,
    pub frequency_ranges: [RangeInclusive<f32>; 2],
}

impl Default for FrequencyTransform {
    fn default() -> Self {
        let bounds_matrix: nalgebra::Matrix2x4<f32> = nalgebra::matrix![
            300_f32, 1000_f32, 1000_f32,  260_f32;
            800_f32, 1400_f32, 2000_f32, 3200_f32];
        let f_ranges: [RangeInclusive<f32>; 2] =
            std::array::from_fn(|i| bounds_matrix.row(i).min()..=bounds_matrix.row(i).max());
        Self {
            bounds_matrix: bounds_matrix,
            frequency_ranges: f_ranges,
        }
    }
}

impl FrequencyTransform {
    pub fn coordinates_to_frequencies(&self, x: f32, y: f32) -> [f32; 2] {
        let xy = x * y;
        let v = self.bounds_matrix * nalgebra::vector![1_f32 - x - y + xy, x - xy, xy, y - xy];
        [v[0], v[1]]
    }

    pub fn gain_db_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        let ratio = self.frequency_ratio(frequency, index);
        if index == 0 {
            12_f32 - ratio * 6_f32
        } else {
            6_f32 + ratio * 6_f32
        }
    }

    pub fn q_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        if index == 0 {
            10_f32
        } else {
            8_f32 + self.frequency_ratio(frequency, index) * 10_f32
        }
    }

    pub fn y_from_x_and_frequencies(&self, x: f32, frequencies: [f32; 2]) -> f32 {
        let m = &self.bounds_matrix;
        let a: [f32; 2] = std::array::from_fn(|i| m.row(i)[0] - m.row(i)[1]);
        let b: [f32; 2] = std::array::from_fn(|i| m.row(i)[0] - m.row(i)[3]);
        let c: [f32; 2] =
            std::array::from_fn(|i| m.row(i)[0] - m.row(i)[1] + m.row(i)[2] - m.row(i)[3]);
        (c[0] * frequencies[1] - c[1] * frequencies[0]
            + (a[1] * c[0] - a[0] * c[1]) * x
            + c[1] * m.row(0)[0]
            - c[0] * m.row(1)[0])
            / (b[0] * c[1] - b[1] * c[0])
    }

    fn frequency_ratio(&self, frequency: f32, index: usize) -> f32 {
        assert!(index < 2);
        ((frequency - self.frequency_ranges[index].start())
            / range::get_length(&self.frequency_ranges[index]))
        .clamp(0_f32, 1_f32)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_bounds() {
        let bound_coords = [
            [0_f32, 0_f32],
            [1_f32, 0_f32],
            [1_f32, 1_f32],
            [0_f32, 1_f32],
        ];
        let transform = FrequencyTransform::default();
        for i in 0..4 {
            let [x, y] = bound_coords[i];
            let bound = transform.bounds_matrix.column(i);
            let frequencies = transform.coordinates_to_frequencies(x, y);
            for j in 0..2 {
                assert_approx_eq!(bound[j], frequencies[j]);
            }
        }
    }

    #[test]
    fn test_y_from_x() {
        let test_coords = [
            [0_f32, 0_f32],
            [0_f32, 0.5_f32],
            [0_f32, 1_f32],
            [0.5_f32, 0_f32],
            [0.5_f32, 0.5_f32],
            [0.5_f32, 1_f32],
            [1_f32, 0_f32],
            [1_f32, 0.5_f32],
            [1_f32, 1_f32],
        ];
        let transform = FrequencyTransform::default();
        for [x, y] in test_coords {
            let frequencies = transform.coordinates_to_frequencies(x, y);
            let y_from_x = transform.y_from_x_and_frequencies(x, frequencies);
            assert_approx_eq!(y, y_from_x);
        }
    }
}
