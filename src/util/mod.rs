use rand::distr::{Alphanumeric, SampleString};

pub fn generate_random_alphanumeric_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), len)
}