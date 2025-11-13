use anyhow::{Context, Result};
use dishaster_models::TrialQARank;

/// Parse question-to-answer ranks from the ranks.txt format.
/// Each question can have multiple keywords, so each line may contain multiple rank lists.
/// Empty lines separate questions.
pub fn parse_qa_ranks(text: &str) -> Result<Vec<Vec<Vec<TrialQARank>>>> {
    let mut all_ranks = Vec::new();
    let mut current_question_ranks = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() {
            // Empty line marks end of current question's keywords
            all_ranks.push(std::mem::take(&mut current_question_ranks));
        } else {
            // Parse a single keyword's ranks
            let ranks = parse_rank_line(line)
                .with_context(|| format!("Failed to parse QA rank line {}", line_num + 1))?;
            current_question_ranks.push(ranks);
        }
    }

    // Don't forget the last question if file doesn't end with empty line
    if !current_question_ranks.is_empty() {
        all_ranks.push(current_question_ranks);
    }

    Ok(all_ranks)
}

/// Parse ranks from the ranks_r.txt format.
/// Each line is a single item's ranks to continued items.
pub fn parse_aq_ranks(text: &str, kind: &str) -> Result<Vec<Vec<TrialQARank>>> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_num, line)| {
            parse_rank_line(line.trim())
                .with_context(|| format!("Failed to parse {kind} rank line {}", line_num + 1))
        })
        .collect()
}

/// Parse a single line of ranks in the format "index:score,index:score,..."
fn parse_rank_line(line: &str) -> Result<Vec<TrialQARank>> {
    line.split(',')
        .map(|entry| {
            let entry = entry.trim();
            let (index_str, score_str) = entry
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("Invalid rank entry format: {entry}"))?;

            let answer_index = index_str
                .parse::<usize>()
                .with_context(|| format!("Invalid index: {index_str}"))?;

            let score = score_str
                .parse::<f32>()
                .with_context(|| format!("Invalid score: {score_str}"))?;

            Ok(TrialQARank {
                answer_index,
                score,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rank_line() {
        let line = "6:0.204658,1:0.048798,17:0.020938";
        let ranks = parse_rank_line(line).unwrap();

        assert_eq!(ranks.len(), 3);
        assert_eq!(ranks[0].answer_index, 6);
        assert!((ranks[0].score - 0.204658).abs() < 1e-6);
        assert_eq!(ranks[1].answer_index, 1);
        assert!((ranks[1].score - 0.048798).abs() < 1e-6);
        assert_eq!(ranks[2].answer_index, 17);
        assert!((ranks[2].score - 0.020938).abs() < 1e-6);
    }

    #[test]
    fn test_parse_aq_ranks() {
        let text = "3:0.229003,2:0.122926,0:0.109921\n9:0.027912,12:0.002898,3:0.001904\n";
        let all_ranks = parse_aq_ranks(text, "AQ").unwrap();

        assert_eq!(all_ranks.len(), 2);
        assert_eq!(all_ranks[0].len(), 3);
        assert_eq!(all_ranks[0][0].answer_index, 3);
        assert_eq!(all_ranks[1].len(), 3);
        assert_eq!(all_ranks[1][0].answer_index, 9);
    }

    #[test]
    fn test_parse_qa_ranks() {
        let text = "6:0.204658,1:0.048798\n7:0.207055,0:0.050373\n\n4:0.040316,0:0.036857\n\n";
        let all_ranks = parse_qa_ranks(text).unwrap();

        assert_eq!(all_ranks.len(), 2);
        // First question has 2 keywords
        assert_eq!(all_ranks[0].len(), 2);
        assert_eq!(all_ranks[0][0].len(), 2);
        assert_eq!(all_ranks[0][0][0].answer_index, 6);
        assert_eq!(all_ranks[0][1].len(), 2);
        assert_eq!(all_ranks[0][1][0].answer_index, 7);
        // Second question has 1 keyword
        assert_eq!(all_ranks[1].len(), 1);
        assert_eq!(all_ranks[1][0].len(), 2);
        assert_eq!(all_ranks[1][0][0].answer_index, 4);
    }
}
