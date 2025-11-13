#!/usr/bin/env python3
"""
Generate QA (Question→Answer) and AQ (Answer→Question) semantic rank mappings.

This script creates the foundational semantic mappings for the trial dialogue system:
- QA ranks: Maps questions to relevant answers using keyword-based retrieval
- AQ ranks: Maps answers to follow-up questions for dialogue flow

Output files:
- ../assets/data/trial/ranks_qa.txt - Question to answer mappings
- ../assets/data/trial/ranks_aq.txt - Answer to question mappings

The ranks use a two-stage approach:
1. First pass: BGE embeddings for fast retrieval (top-20)
2. Second pass: Cross-encoder reranking for accuracy (top-10)
"""

import re
import tomllib
from pathlib import Path
from typing import cast

import torch
from sentence_transformers import CrossEncoder, SentenceTransformer
from tqdm import tqdm

# Configuration
FIRST_PASS_TOP_K = 20  # Initial retrieval count
SECOND_PASS_TOP_K = 10  # Final reranked count


def load_models() -> tuple[SentenceTransformer, CrossEncoder]:
    """Load embedding and reranking models."""
    print("Loading embedding model...")
    embedder = SentenceTransformer(
        "BAAI/bge-large-zh-v1.5",
        # local_files_only=True,
    )

    print("Loading reranking model...")
    reranker = CrossEncoder(
        "BAAI/bge-reranker-v2-m3",
        # local_files_only=True,
    )

    return embedder, reranker


def load_corpus(corpus_root: Path) -> tuple[list[str], list[str], list[dict]]:
    """Load trial corpus data."""
    print("Loading corpus...")
    raw_corpus_r = tomllib.loads((corpus_root / "corpus_r.toml").read_text(encoding="utf-8"))[
        "item"
    ]
    raw_corpus = tomllib.loads((corpus_root / "corpus.toml").read_text(encoding="utf-8"))["item"]

    # Responses (answers)
    corpus = [v["text"].replace("\\", "") for v in raw_corpus_r]

    # Questions (remove keyword markers for embedding)
    queries = [v["text"].replace("[", "").replace("]", "").replace("\\", "") for v in raw_corpus]

    return corpus, queries, raw_corpus


def generate_qa_ranks(
    embedder: SentenceTransformer,
    reranker: CrossEncoder,
    corpus: list[str],
    queries: list[str],
    raw_corpus: list[dict],
    corpus_root: Path,
) -> None:
    """
    Generate Question→Answer ranks with keyword-based retrieval.

    For each question:
    1. Extract keywords from question text (marked with [])
    2. For each keyword, find relevant answers using semantic search
    3. Rerank using cross-encoder with keyword context
    4. Output top-10 answers per keyword
    """
    print("\n" + "=" * 70)
    print("Generating Question→Answer (QA) ranks...")
    print("=" * 70)

    # Pre-encode all answers
    corpus_embeddings = cast(torch.Tensor, embedder.encode_document(corpus, convert_to_tensor=True))

    all_ranks: list[list[list[tuple[int, float]]]] = []

    for i, query in tqdm(enumerate(queries), total=len(queries), desc="Processing questions"):
        # Extract keywords from original text (with [keyword] markers)
        keywords: list[str] = re.findall(r"\[(.+?)\]", raw_corpus[i]["text"])

        if not keywords:
            print(f"\nWarning: No keywords found for question {i}: {query[:50]}...")

        result_ranks: list[list[tuple[int, float]]] = []

        for kw in keywords:
            # Encode query with keyword-specific prompt
            kw_embedding = cast(
                torch.Tensor,
                embedder.encode_query(
                    query,
                    prompt=f"为这个句子生成表示以用于检索合适的回答（关键词：{kw}）：",
                    convert_to_tensor=True,
                ),
            )

            # First pass: fast similarity search
            kw_similarity_scores = embedder.similarity(kw_embedding, corpus_embeddings)[0]
            first_pass_k = min(FIRST_PASS_TOP_K, len(corpus))
            _scores, indices = torch.topk(kw_similarity_scores, k=first_pass_k)

            # Second pass: rerank with cross-encoder
            passages = [corpus[idx] for idx in indices]
            ranks = reranker.rank(query + f"（关键词：{kw}）", passages)

            # Store top results
            second_pass_k = min(SECOND_PASS_TOP_K, len(ranks))
            result_ranks.append(
                [
                    (int(indices[int(rank["corpus_id"])]), float(rank["score"]))
                    for rank in ranks[:second_pass_k]
                ]
            )

        all_ranks.append(result_ranks)

    # Format output: one line per keyword, empty line between questions
    result_text = "\n".join(
        "".join(
            ",".join(f"{cid}:{score:.6f}" for cid, score in ranks) + "\n" for ranks in result_ranks
        )
        for result_ranks in all_ranks
    )

    output_path = corpus_root / "ranks_qa.txt"
    output_path.write_text(result_text, encoding="utf-8", newline="\n")
    print(f"✓ Saved QA ranks to {output_path}")


def generate_aq_ranks(
    embedder: SentenceTransformer,
    reranker: CrossEncoder,
    corpus: list[str],
    queries: list[str],
    corpus_root: Path,
) -> None:
    """
    Generate Answer→Question ranks for dialogue flow.

    For each answer:
    1. Find semantically related follow-up questions
    2. Rerank using cross-encoder
    3. Output top-10 questions

    Used to select relevant questions after player responds.
    """
    print("\n" + "=" * 70)
    print("Generating Answer→Question (AQ) ranks...")
    print("=" * 70)

    # Pre-encode all questions
    corpus_embeddings = cast(
        torch.Tensor, embedder.encode_document(queries, convert_to_tensor=True)
    )

    all_ranks: list[list[tuple[int, float]]] = []

    for query in tqdm(corpus, total=len(corpus), desc="Processing answers"):
        # Encode answer
        query_embedding = cast(torch.Tensor, embedder.encode_query(query, convert_to_tensor=True))

        # First pass: fast similarity search
        similarity_scores = embedder.similarity(query_embedding, corpus_embeddings)[0]
        first_pass_k = min(FIRST_PASS_TOP_K, len(queries))
        _scores, indices = torch.topk(similarity_scores, k=first_pass_k)

        # Second pass: rerank with cross-encoder
        # For AQ ranks, no keywords needed - just semantic relevance
        passages = [queries[idx] for idx in indices]
        ranks = reranker.rank(query, passages)

        # Store top results
        second_pass_k = min(SECOND_PASS_TOP_K, len(ranks))
        all_ranks.append(
            [
                (int(indices[int(rank["corpus_id"])]), float(rank["score"]))
                for rank in ranks[:second_pass_k]
            ]
        )

    # Format output: one line per answer
    result_text = "\n".join(
        ",".join(f"{cid}:{score:.6f}" for cid, score in ranks) for ranks in all_ranks
    )

    output_path = corpus_root / "ranks_aq.txt"
    output_path.write_text(result_text, encoding="utf-8", newline="\n")
    print(f"✓ Saved AQ ranks to {output_path}")


def main() -> None:
    """Generate QA and AQ ranks."""
    corpus_root = Path("../assets/data/trial")

    # Load models and data
    embedder, reranker = load_models()
    corpus, queries, raw_corpus = load_corpus(corpus_root)

    print(f"\nLoaded {len(queries)} questions and {len(corpus)} answers")

    # Generate ranks
    generate_qa_ranks(embedder, reranker, corpus, queries, raw_corpus, corpus_root)
    generate_aq_ranks(embedder, reranker, corpus, queries, corpus_root)

    print("\n" + "=" * 70)
    print("QA/AQ rank generation complete!")
    print("=" * 70)


if __name__ == "__main__":
    main()
