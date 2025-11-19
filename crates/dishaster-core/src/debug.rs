use dishaster_models::{ReputationConfig, ReputationState};

/// Get formatted feedback statistics summary
pub fn format_feedback_stats(
    reputation: &ReputationState,
    config: &ReputationConfig,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "\n=== Feedback Statistics ===")?;
    writeln!(
        &mut output,
        "{:<12} {:>8} {:>12} {:>10} {:>10}",
        "Topic", "Triggers", "Total Impact", "Base", "Impact%"
    )?;
    writeln!(&mut output, "{:-<62}", "")?;

    for (topic, stats) in &reputation.feedback_stats {
        if stats.trigger_count > 0 {
            let base_impact = config.base_impacts[topic];
            let impact_prob = config.impact_probabilities[topic];
            writeln!(
                &mut output,
                "{:<12} {:>8} {:>12.2} {:>10.2} {:>9.0}%",
                format!("{:?}", topic),
                stats.trigger_count,
                stats.total_reputation_impact,
                base_impact,
                impact_prob * 100.0
            )?;
        }
    }

    writeln!(&mut output, "{:-<62}", "")?;
    writeln!(
        &mut output,
        "Total accumulated: {:.2}",
        reputation.daily_accumulated
    )?;

    Ok(output)
}
