use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

#[derive(Resource)]
pub struct GameRng {
    pub rng: SmallRng,
}

impl Default for GameRng {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }
}

impl GameRng {
    pub fn gen_range<T>(&mut self, range: std::ops::RangeInclusive<T>) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
    {
        self.rng.gen_range(range)
    }
    
    pub fn gen<T>(&mut self) -> T
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        self.rng.gen()
    }
}
