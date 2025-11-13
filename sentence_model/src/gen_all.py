"""
Generate all trial system features in correct order.

This is the main entry point for generating all NLP features for the trial system:
1. QA/AQ ranks - Question-answer semantic mappings
2. Sentiment scores - Response quality evaluation
3. QQ/RR ranks - Multi-turn dialogue continuations

Usage:
    uv run src/gen_all.py
"""

import sys


def main() -> int:
    """Generate all features in dependency order."""
    print("=" * 70)
    print("Trial System NLP Feature Generation")
    print("=" * 70)
    print()

    # Step 1: Generate QA/AQ ranks
    print("[1/3] Generating QA/AQ ranks...")
    print("-" * 70)
    try:
        from gen_qa_aq_ranks import main as gen_qa_aq_main

        gen_qa_aq_main()
        print("✓ QA/AQ ranks generated successfully")
    except Exception as e:
        print(f"✗ Error generating QA/AQ ranks: {e}")
        return 1
    print()

    # Step 2: Generate sentiment scores
    print("[2/3] Generating sentiment scores...")
    print("-" * 70)
    try:
        from gen_sentiment_scores import main as gen_sentiment_main

        gen_sentiment_main()
        print("✓ Sentiment scores generated successfully")
    except Exception as e:
        print(f"✗ Error generating sentiment scores: {e}")
        return 1
    print()

    # Step 3: Generate QQ/RR continuation ranks
    print("[3/3] Generating QQ/RR continuation ranks...")
    print("-" * 70)
    try:
        from gen_continuation_ranks import main as gen_continuation_main

        gen_continuation_main()
        print("✓ QQ/RR ranks generated successfully")
    except Exception as e:
        print(f"✗ Error generating continuation ranks: {e}")
        return 1
    print()

    print("=" * 70)
    print("All features generated successfully! 🎉")
    print("=" * 70)
    print()
    print("Generated files:")
    print("  - ../assets/data/trial/ranks_qa.txt")
    print("  - ../assets/data/trial/ranks_aq.txt")
    print("  - ../assets/data/trial/ranks_qq.txt")
    print("  - ../assets/data/trial/ranks_rr.txt")
    print("  - ../assets/data/trial/corpus_r.toml (updated with scores)")
    print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
