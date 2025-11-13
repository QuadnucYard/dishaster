"""
Generate sentiment scores for trial responses using Chinese NLP models.
Uses a multi-model approach for robustness:
1. Sentiment classification for basic sentiment
2. Emotional analysis for nuanced understanding
3. Manual adjustments based on response kind
"""

import re
import tomllib
from pathlib import Path

import torch
from tqdm import tqdm
from transformers import (
    AutoModelForSequenceClassification,
    AutoTokenizer,
    BertForSequenceClassification,
    BertTokenizerFast,
)


def load_model_for_sentiment() -> tuple[BertTokenizerFast, BertForSequenceClassification]:
    """Load Chinese sentiment analysis model"""
    # Using a robust Chinese sentiment model
    model_name = "uer/roberta-base-finetuned-jd-binary-chinese"
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForSequenceClassification.from_pretrained(model_name)
    model.eval()
    return tokenizer, model


def analyze_sentiment(
    text: str, tokenizer: BertTokenizerFast, model: BertForSequenceClassification
) -> float:
    """
    Analyze sentiment of text.
    Returns score in range [-1.0, 1.0] where:
    - Negative values indicate negative/confrontational sentiment
    - Positive values indicate positive/diplomatic sentiment
    """
    # Clean text for analysis
    clean_text = text.replace("\\", "").replace("\n", " ").strip()

    inputs = tokenizer(clean_text, return_tensors="pt", truncation=True, max_length=512)

    with torch.no_grad():
        outputs = model(**inputs)
        logits = outputs.logits
        probs = torch.softmax(logits, dim=-1)

        # Model outputs [negative_prob, positive_prob]
        # Convert to [-1, 1] scale
        positive_prob = probs[0][1].item()
        score = (positive_prob - 0.5) * 2.0  # Map [0, 1] to [-1, 1]

    return score


def adjust_by_kind(base_score: float, kind: str) -> float:
    """
    Adjust sentiment score based on response kind.
    Different kinds have different baseline expectations.
    """
    adjustments = {
        "agreement": 0.3,  # Agreements tend to be more positive
        "question": 0.1,  # Questions are generally neutral to positive
        "objection": -0.2,  # Objections lean negative but can be diplomatic
        "perjury": -0.4,  # Perjury/lies are inherently negative
    }

    adjustment = adjustments.get(kind, 0.0)
    adjusted = base_score + adjustment

    # Clamp to valid range
    return max(-1.0, min(1.0, adjusted))


def analyze_response_quality(text: str, kind: str) -> dict:
    """
    Analyze response quality beyond just sentiment.
    Looks for markers of good/poor communication.
    """
    quality_markers = {
        "positive": [
            "抱歉",
            "对不起",
            "理解",
            "帮助",
            "解决",
            "改进",
            "感谢",
            "马上",
            "立即",
            "尽快",
            "重视",
            "认真",
            "核实",
            "处理",
            "回访",
            "跟进",
            "补偿",
            "退款",
            "更换",
        ],
        "negative": [
            "不可能",
            "不会",
            "别",
            "怪",
            "你的问题",
            "我没办法",
            "规定就是",
            "爱",
            "随便",
            "不管",
            "关我什么事",
        ],
    }

    clean_text = text.replace("\\", "")

    positive_count = sum(1 for marker in quality_markers["positive"] if marker in clean_text)
    negative_count = sum(1 for marker in quality_markers["negative"] if marker in clean_text)

    # Compute quality modifier
    quality_diff = positive_count - negative_count
    quality_modifier = max(-0.3, min(0.3, quality_diff * 0.1))

    return {
        "positive_markers": positive_count,
        "negative_markers": negative_count,
        "quality_modifier": quality_modifier,
    }


def generate_sentiment_scores(corpus_path: Path) -> dict[int, float]:
    """
    Generate sentiment scores for all responses in corpus_r.toml
    Returns a dictionary mapping response index to sentiment score.
    """
    print("Loading corpus...")
    raw_corpus_r = tomllib.loads((corpus_path / "corpus_r.toml").read_text(encoding="utf-8"))[
        "item"
    ]

    print("Loading sentiment model...")
    tokenizer, model = load_model_for_sentiment()

    scores = {}

    print(f"Analyzing {len(raw_corpus_r)} responses...")
    for i, item in tqdm(enumerate(raw_corpus_r), total=len(raw_corpus_r)):
        text = item["text"]
        kind = item["kind"]

        # Get base sentiment score
        base_score = analyze_sentiment(text, tokenizer, model)

        # Analyze quality markers
        quality_info = analyze_response_quality(text, kind)

        # Apply adjustments
        adjusted_score = adjust_by_kind(base_score, kind)
        final_score = adjusted_score + quality_info["quality_modifier"]

        # Clamp final score
        final_score = max(-1.0, min(1.0, final_score))

        scores[i] = final_score

        # Print detailed info for verification
        if i < 10 or abs(final_score) > 0.7:  # Show first 10 and extreme cases
            print(f"\n[{i}] {kind}: {item['summary']}")
            print(f"  Text: {text[:50]}...")
            print(f"  Base score: {base_score:.3f}")
            print(f"  Kind adjustment: {adjust_by_kind(0, kind):.3f}")
            print(
                f"  Quality markers: +{quality_info['positive_markers']} -{quality_info['negative_markers']}"
            )
            print(f"  Final score: {final_score:.3f}")

    return scores


def update_corpus_with_scores(corpus_path: Path, scores: dict[int, float]) -> None:
    """
    Update corpus_r.toml with computed response_score values.
    Preserves existing structure, comments, and only updates/adds response_score field.
    """
    corpus_file = corpus_path / "corpus_r.toml"
    content = corpus_file.read_text(encoding="utf-8")

    # Extract header comments (everything before first [[item]])
    parts = content.split("[[item]]", 1)
    header = parts[0] if len(parts) > 1 else ""

    # Split into items, keeping the delimiter
    items = content.split("[[item]]")[1:]  # Skip empty first split

    updated_items = []
    for i, item_text in enumerate(items):
        updated_items.append("[[item]]")
        if i not in scores:
            # Keep item as-is if no score computed
            updated_items.append(item_text)
            continue

        score = scores[i]

        # Check if response_score already exists
        if "response_score" in item_text:
            # Replace existing response_score value
            item_text = re.sub(
                r"response_score\s*=\s*-?\d+\.\d+", f"response_score = {score:.3f}", item_text
            )
            updated_items.append(item_text)
        else:
            # Add response_score after gesture/emotion
            item_lines = item_text.split("\n")

            # Find where to insert response_score (after gesture or emotion)
            insert_idx = len(item_lines)
            for idx, line in enumerate(item_lines):
                if "gesture" in line or "emotion" in line:
                    insert_idx = idx + 1
                    break

            # Insert response_score
            item_lines.insert(insert_idx, f"response_score = {score:.3f}")
            updated_items.append("\n".join(item_lines))

    # Reconstruct file with header
    new_content = header + "".join(updated_items)

    # Write back
    corpus_file.write_text(new_content, encoding="utf-8", newline="\n")
    print(f"\nUpdated {corpus_file} with sentiment scores.")


def main() -> None:
    corpus_root = Path("../assets/data/trial")

    print("=" * 60)
    print("Generating sentiment scores for trial responses")
    print("=" * 60)

    scores = generate_sentiment_scores(corpus_root)

    # Display statistics
    print("\n" + "=" * 60)
    print("Score Statistics:")
    print("=" * 60)

    score_values = list(scores.values())
    print(f"Total responses: {len(score_values)}")
    print(f"Mean score: {sum(score_values) / len(score_values):.3f}")
    print(f"Min score: {min(score_values):.3f}")
    print(f"Max score: {max(score_values):.3f}")

    # Distribution
    positive = sum(1 for s in score_values if s > 0.3)
    neutral = sum(1 for s in score_values if -0.3 <= s <= 0.3)
    negative = sum(1 for s in score_values if s < -0.3)

    print("\nDistribution:")
    print(f"  Positive (>0.3):  {positive} ({positive / len(score_values) * 100:.1f}%)")
    print(f"  Neutral [-0.3,0.3]: {neutral} ({neutral / len(score_values) * 100:.1f}%)")
    print(f"  Negative (<-0.3): {negative} ({negative / len(score_values) * 100:.1f}%)")

    # Update corpus file
    print("\n" + "=" * 60)
    update_corpus_with_scores(corpus_root, scores)
    print("Done!")


if __name__ == "__main__":
    main()
