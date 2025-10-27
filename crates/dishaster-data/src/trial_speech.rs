use dishaster_models::{TrialSpeech, TrialSpeechItem};
use thiserror::Error;

/// An error that occurred while parsing trial speech items.
#[derive(Debug, Error)]
pub enum TrailSpeechParseError {
    /// An unclosed keyword was found in the speech text.
    #[error("Unclosed keyword in speech text")]
    UnclosedKeyword,
}

/// Populate `TrialSpeech` items.
pub fn populate_trial_speech_items(
    speeches: &mut [TrialSpeech],
) -> Result<(), TrailSpeechParseError> {
    for speech in speeches {
        hydrate_speech_items(speech)?;
    }
    Ok(())
}

/// Populate `TrialResponse` items.
pub fn populate_trial_response_items(
    responses: &mut [dishaster_models::TrialResponse],
) -> Result<(), TrailSpeechParseError> {
    for response in responses {
        hydrate_speech_items(&mut response.content)?;
    }
    Ok(())
}

fn hydrate_speech_items(speech: &mut TrialSpeech) -> Result<(), TrailSpeechParseError> {
    speech.items = parse_items(&speech.text)?;
    Ok(())
}

fn parse_items(text: &str) -> Result<Vec<TrialSpeechItem>, TrailSpeechParseError> {
    let mut items = Vec::new();
    let mut start = 0;
    let mut char_indices = text.char_indices().peekable();

    while let Some((i, ch)) = char_indices.next() {
        match ch {
            '[' => {
                if start < i {
                    items.push(TrialSpeechItem::Text(text[start..i].into()));
                }
                let keyword_start = i + ch.len_utf8();
                let mut closed = false;
                while let Some((j, next_ch)) = char_indices.peek().copied() {
                    if next_ch == ']' {
                        char_indices.next(); // consume ]
                        items.push(TrialSpeechItem::Keyword(text[keyword_start..j].into()));
                        closed = true;
                        start = j + next_ch.len_utf8();
                        break;
                    } else {
                        char_indices.next();
                    }
                }
                if !closed {
                    return Err(TrailSpeechParseError::UnclosedKeyword);
                }
            }
            '\\' => {
                if start < i {
                    items.push(TrialSpeechItem::Text(text[start..i].into()));
                }
                items.push(TrialSpeechItem::LineBreak);
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }

    if start < text.len() {
        items.push(TrialSpeechItem::Text(text[start..].into()));
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let text = "hello world";
        let items = parse_items(text).unwrap();
        assert_eq!(items, vec![TrialSpeechItem::Text("hello world".into())]);
    }

    #[test]
    fn test_parse_with_keyword() {
        let text = "this is [keyword]";
        let items = parse_items(text).unwrap();
        assert_eq!(
            items,
            vec![
                TrialSpeechItem::Text("this is ".into()),
                TrialSpeechItem::Keyword("keyword".into()),
            ]
        );
    }

    #[test]
    fn test_parse_with_line_break() {
        let text = "line1\\line2";
        let items = parse_items(text).unwrap();
        assert_eq!(
            items,
            vec![
                TrialSpeechItem::Text("line1".into()),
                TrialSpeechItem::LineBreak,
                TrialSpeechItem::Text("line2".into()),
            ]
        );
    }

    #[test]
    fn test_parse_mixed() {
        let text = "开始 [关键词]\\end";
        let items = parse_items(text).unwrap();
        assert_eq!(
            items,
            vec![
                TrialSpeechItem::Text("开始 ".into()),
                TrialSpeechItem::Keyword("关键词".into()),
                TrialSpeechItem::LineBreak,
                TrialSpeechItem::Text("end".into()),
            ]
        );
    }

    #[test]
    fn test_parse_unclosed_keyword() {
        let text = "hello [unclosed";
        let result = parse_items(text);
        assert!(matches!(
            result,
            Err(TrailSpeechParseError::UnclosedKeyword)
        ));
    }

    #[test]
    fn test_parse_empty_keyword() {
        let text = "hello [] world";
        let items = parse_items(text).unwrap();
        assert_eq!(
            items,
            vec![
                TrialSpeechItem::Text("hello ".into()),
                TrialSpeechItem::Keyword("".into()),
                TrialSpeechItem::Text(" world".into()),
            ]
        );
    }

    #[test]
    fn test_parse_multiple_keywords() {
        let text = "[a] and [b]";
        let items = parse_items(text).unwrap();
        assert_eq!(
            items,
            vec![
                TrialSpeechItem::Keyword("a".into()),
                TrialSpeechItem::Text(" and ".into()),
                TrialSpeechItem::Keyword("b".into()),
            ]
        );
    }
}
