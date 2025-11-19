//! Responses from the simulation to the client.

/// Responses that can be sent from the simulation to the client.
pub enum SimResponse {
    /// Response to a distance query.
    Distance(Option<f32>),
    /// Response to a distance field query.
    Distances(Box<DistancesResponse>),

    /// Response containing feedback statistics for debugging purposes.
    FeedbackStats(String),
}

/// Response to a distance field query.
#[derive(Debug, Clone)]
pub struct DistancesResponse {
    /// Width of the grid in cells.
    pub width: usize,
    /// Height of the grid in cells.
    pub height: usize,
    /// Size of each cell in world units.
    pub cell_size: f32,
    /// Flattened col-major grid of distances.
    pub data: Vec<f32>,
}
