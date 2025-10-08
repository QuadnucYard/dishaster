use crate::prelude::*;

/// A navigation path consisting of waypoints.
#[derive(Debug, Default, Clone)]
pub struct NavPath {
    /// Waypoints in the path, ordered from goal to start.
    pub waypoints: Vec<Vec2>,
}

impl NavPath {
    /// Create a new navigation path from a list of waypoints.
    /// The waypoints are expected to be in order from start to goal.
    pub fn new(mut waypoints: Vec<Vec2>) -> Self {
        waypoints.reverse();
        Self { waypoints }
    }

    /// Get the number of waypoints in the path.
    #[inline]
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Check if the path is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// Clear all waypoints from the path.
    #[inline]
    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    /// Get the next waypoint.
    #[inline]
    pub fn next(&self) -> Option<Vec2> {
        self.waypoints.last().copied()
    }

    /// Get the last waypoint.
    #[inline]
    pub fn last(&self) -> Option<Vec2> {
        self.waypoints.first().copied()
    }

    /// Pop and return the next waypoint from the path.
    #[inline]
    pub fn pop(&mut self) -> Option<Vec2> {
        self.waypoints.pop()
    }

    /// Push a new waypoint to the front of the path.
    #[inline]
    pub fn push(&mut self, waypoint: Vec2) {
        self.waypoints.push(waypoint);
    }
}
