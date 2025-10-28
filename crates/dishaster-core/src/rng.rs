//! Pseudorandom number generators for deterministic simulation

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use rand_xoshiro::Xoshiro256PlusPlus;

use crate::prelude::*;

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
