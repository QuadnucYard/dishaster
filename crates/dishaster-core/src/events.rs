use crate::prelude::*;

/// Event signaling the start of a run
#[derive(Event)]
pub struct RunStarted;

/// Event signaling the end of a run
#[derive(Event)]
pub struct RunEnded;
