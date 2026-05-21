pub trait ConfigTrait {
    const NUM_BANDS: usize;
    const NUM_CHANNELS: usize;
    const ANALYZER_NUM_BINS: usize;
}

pub struct Config;

impl ConfigTrait for Config {
    const NUM_BANDS: usize = 8;
    const NUM_CHANNELS: usize = 2;
    const ANALYZER_NUM_BINS: usize = 12;
}
