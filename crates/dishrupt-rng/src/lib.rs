//! Pseudorandom number generators for deterministic simulation

pub mod prelude;

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use rand::{Rng, RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

/// Pseudorandom number generator for deterministic simulation
#[derive(Deref, DerefMut)]
pub struct Prng(Xoshiro256PlusPlus);

impl Prng {
    /// Create a new deterministic RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        Self(SeedableRng::seed_from_u64(seed))
    }

    /// Generate a random seed
    pub fn derive_seed(&mut self) -> u64 {
        self.random()
    }

    /// Derive a new Prng from this one
    pub fn derive_prng(&mut self) -> Prng {
        Prng::new(self.derive_seed())
    }
}

/// Entity-specific pseudorandom number generator component
#[derive(Component, Deref, DerefMut)]
pub struct EntityRng(Prng);

impl EntityRng {
    /// Create a new entity RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        Self(Prng::new(seed))
    }
}

/// Game-wide pseudorandom number generator resource
#[derive(Resource, Deref, DerefMut)]
pub struct WorldRng(Prng);

impl WorldRng {
    /// Create a new world RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        Self(Prng::new(seed))
    }
}

/// Tagged RNG resource for navigation-related randomness
#[derive(Resource)]
pub struct SystemRng<Tag>(Prng, PhantomData<Tag>);

impl<Tag> SystemRng<Tag> {
    /// Create a new RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        Self(Prng::new(seed), PhantomData)
    }
}

impl<Tag> Deref for SystemRng<Tag> {
    type Target = Prng;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Tag> DerefMut for SystemRng<Tag> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// === Extensions ===

/// Extension trait for RNG with delta-time-aware random booleans
pub trait RandomDt {
    /// Use exponential probability: P(detect in dt) = 1 - exp(-rate * dt)
    /// This ensures the overall detection probability is independent of tick rate
    ///
    /// Convert rate to probability for this tick (exponential distribution)
    /// For small dt, this approximates to detection_rate * dt, but is mathematically correct
    fn random_bool_dt(&mut self, p: f64, dt: f64) -> bool;
}

impl<T: Rng> RandomDt for T {
    #[inline]
    fn random_bool_dt(&mut self, p: f64, dt: f64) -> bool {
        self.random_bool(1.0 - (-p * dt).exp())
    }
}
