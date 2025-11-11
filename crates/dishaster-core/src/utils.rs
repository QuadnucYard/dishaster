use dishrupt_rng::prelude::Rng;

/// Sigmoid function for mapping unbounded values to 0..1
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Softmax sampling from scores
///
/// Returns index of selected item based on softmax probabilities.
pub fn sample_softmax(scores: &[f32], temperature: f32, rng: &mut impl Rng) -> Option<usize> {
    if scores.is_empty() {
        return None;
    }

    // Apply temperature and find max for numerical stability
    let max_score = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    // Compute exp(score/T - max/T) for stability
    let exp_scores: Vec<f32> = scores
        .iter()
        .map(|&s| ((s - max_score) / temperature).exp())
        .collect();

    let sum: f32 = exp_scores.iter().sum();

    if sum <= 0.0 {
        return None;
    }

    // Sample using cumulative distribution
    let threshold = rng.random_range(0.0..sum);
    let mut cumulative = 0.0;

    for (i, &exp_score) in exp_scores.iter().enumerate() {
        cumulative += exp_score;
        if cumulative >= threshold {
            return Some(i);
        }
    }

    // Fallback (should not reach here)
    Some(scores.len() - 1)
}
