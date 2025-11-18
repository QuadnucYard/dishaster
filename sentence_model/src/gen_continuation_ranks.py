"""
Generate continuation scores for trial dialogue flow.
Supports multi-turn speech by computing how well one statement follows another.

This enables:
1. Diner continuing their complaint with multiple statements
2. Player giving multi-part responses
3. Natural dialogue flow without forced alternation
"""

import tomllib
from pathlib import Path
from typing import cast

import torch
from sentence_transformers import CrossEncoder, SentenceTransformer
from tqdm import tqdm

from binencode import encode_aq_ranks_bincode


def load_models() -> tuple[SentenceTransformer, CrossEncoder]:
    """Load embedding and reranking models for Chinese text"""
    embedder = SentenceTransformer(
        "BAAI/bge-large-zh-v1.5",
        # local_files_only=True,
    )

    reranker = CrossEncoder(
        "BAAI/bge-reranker-v2-m3",
        # local_files_only=True,
    )

    return embedder, reranker


def generate_qq_ranks(corpus_path: Path, top_k: int = 10) -> list[list[tuple[int, float]]]:
    """
    Generate Question-to-Question (QQ) continuation ranks.
    Determines which diner statements can naturally follow each other.

    Returns: For each question, a ranked list of (question_idx, score) tuples
    indicating which questions can follow it in a multi-turn dialogue.
    """
    print("Loading question corpus...")
    raw_corpus = tomllib.loads((corpus_path / "corpus.toml").read_text(encoding="utf-8"))["item"]
    queries = [v["text"].replace("[", "").replace("]", "").replace("\\", "") for v in raw_corpus]

    print("Loading models...")
    embedder, reranker = load_models()

    print("Encoding questions...")
    corpus_embeddings = cast(
        torch.Tensor, embedder.encode_document(queries, convert_to_tensor=True)
    )

    all_ranks: list[list[tuple[int, float]]] = []

    print(f"Computing continuation scores for {len(queries)} questions...")
    for i, query in tqdm(enumerate(queries), total=len(queries)):
        # Encode current question as "previous statement"
        query_embedding = cast(
            torch.Tensor,
            embedder.encode_query(
                query,
                prompt="为上一句话生成表示以用于检索自然的后续对话：",
                convert_to_tensor=True,
            ),
        )

        # Find semantically similar questions (potential continuations)
        similarity_scores = embedder.similarity(query_embedding, corpus_embeddings)[0]

        # Get top candidates (excluding self)
        _scores, indices = torch.topk(similarity_scores, k=min(top_k * 2, len(queries)))

        # Filter out self
        filtered_indices = [idx for idx in indices if idx != i][: top_k * 2]

        # Rerank using cross-encoder for better quality
        passages = [queries[idx] for idx in filtered_indices]
        continuation_prompt = f"{query}\n[继续]"  # Indicates continuation

        ranks = reranker.rank(continuation_prompt, passages)

        # Store top results
        result = [
            (int(filtered_indices[int(rank["corpus_id"])]), float(rank["score"]))
            for rank in ranks[:top_k]
        ]

        all_ranks.append(result)

        # Show example for first few
        if i < 3:
            print(f"\n[{i}] {query[:40]}...")
            print("  Top continuations:")
            for idx, score in result[:3]:
                print(f"    [{idx}] {score:.3f}: {queries[idx][:40]}...")

    return all_ranks


def generate_rr_ranks(corpus_path: Path, top_k: int = 10) -> list[list[tuple[int, float]]]:
    """
    Generate Response-to-Response (RR) continuation ranks.
    Determines which player responses can naturally follow each other.

    Returns: For each response, a ranked list of (response_idx, score) tuples
    indicating which responses can follow it in a multi-turn reply.
    """
    print("\nLoading response corpus...")
    raw_corpus_r = tomllib.loads((corpus_path / "corpus_r.toml").read_text(encoding="utf-8"))[
        "item"
    ]
    responses = [v["text"].replace("\\", "") for v in raw_corpus_r]

    print("Loading models...")
    embedder, reranker = load_models()

    print("Encoding responses...")
    corpus_embeddings = cast(
        torch.Tensor, embedder.encode_document(responses, convert_to_tensor=True)
    )

    all_ranks: list[list[tuple[int, float]]] = []

    print(f"Computing continuation scores for {len(responses)} responses...")
    for i, response in tqdm(enumerate(responses), total=len(responses)):
        # Encode current response as "previous statement"
        response_embedding = cast(
            torch.Tensor,
            embedder.encode_query(
                response,
                prompt="为上一句回复生成表示以用于检索自然的后续回复：",
                convert_to_tensor=True,
            ),
        )

        # Find semantically similar responses (potential continuations)
        similarity_scores = embedder.similarity(response_embedding, corpus_embeddings)[0]

        # Get top candidates (excluding self)
        _scores, indices = torch.topk(similarity_scores, k=min(top_k * 2, len(responses)))

        # Filter out self
        filtered_indices = [idx for idx in indices if idx != i][: top_k * 2]

        # Rerank using cross-encoder
        passages = [responses[idx] for idx in filtered_indices]
        continuation_prompt = f"{response}\n[继续]"  # Indicates continuation

        ranks = reranker.rank(continuation_prompt, passages)

        # Store top results
        result = [
            (int(filtered_indices[int(rank["corpus_id"])]), float(rank["score"]))
            for rank in ranks[:top_k]
        ]

        all_ranks.append(result)

        # Show example for first few
        if i < 3:
            print(f"\n[{i}] {response[:40]}...")
            print("  Top continuations:")
            for idx, score in result[:3]:
                print(f"    [{idx}] {score:.3f}: {responses[idx][:40]}...")

    return all_ranks


def save_ranks(ranks: list[list[tuple[int, float]]], output_path: Path) -> None:
    """Save ranks in both bincode (production) and txt (debug) formats"""
    # Save bincode format for production
    bincode_data = encode_aq_ranks_bincode(ranks)
    bincode_path = output_path.with_suffix(".bin")
    bincode_path.write_bytes(bincode_data)
    print(f"\n✓ Saved ranks (bincode) to {bincode_path}")

    # Save txt format for debugging (local to sentence_model)
    result_text = "\n".join(
        ",".join(f"{idx}:{score:.6f}" for idx, score in rank_list) for rank_list in ranks
    )
    debug_path = Path("debug_output") / output_path.name
    debug_path.parent.mkdir(exist_ok=True)
    debug_path.write_text(result_text, encoding="utf-8", newline="\n")
    print(f"✓ Saved ranks (txt debug) to {debug_path}")


def main() -> None:
    corpus_root = Path("../assets/data/trial")

    print("=" * 70)
    print("Generating Continuation Ranks for Multi-Turn Dialogue")
    print("=" * 70)

    # Generate QQ ranks (question -> question continuations)
    print("\n" + "=" * 70)
    print("PART 1: Question-to-Question Continuations (Diner multi-turn)")
    print("=" * 70)
    qq_ranks = generate_qq_ranks(corpus_root, top_k=10)
    save_ranks(qq_ranks, corpus_root / "ranks_qq.txt")

    # Generate RR ranks (response -> response continuations)
    print("\n" + "=" * 70)
    print("PART 2: Response-to-Response Continuations (Player multi-turn)")
    print("=" * 70)
    rr_ranks = generate_rr_ranks(corpus_root, top_k=10)
    save_ranks(rr_ranks, corpus_root / "ranks_rr.txt")

    print("\n" + "=" * 70)
    print("Done! Generated continuation ranks for multi-turn dialogue.")
    print("=" * 70)
    print("\nFiles created:")
    print(f"  - {corpus_root / 'ranks_qq.bin'} (question continuations - bincode)")
    print(f"  - {corpus_root / 'ranks_rr.bin'} (response continuations - bincode)")
    print("  - debug_output/ranks_qq.txt (debug text format)")
    print("  - debug_output/ranks_rr.txt (debug text format)")
    print("\nThese enable natural multi-turn conversations without forced alternation.")


if __name__ == "__main__":
    main()
