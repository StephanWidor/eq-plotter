use super::*;

pub fn to_range_inclusive(float_range: &nice::FloatRange) -> std::ops::RangeInclusive<f32> {
    float_range.unnormalize(0_f32)..=float_range.unnormalize(1_f32)
}

pub fn get_length(range: &std::ops::RangeInclusive<f32>) -> f32 {
    *range.end() - *range.start()
}
