from pathlib import Path
import re
import tomllib
from typing import cast
import torch

from sentence_transformers import SentenceTransformer, CrossEncoder
from tqdm import tqdm


embedder = SentenceTransformer(
    "BAAI/bge-large-zh-v1.5",
    # local_files_only=True,
)

reranker = CrossEncoder(
    "BAAI/bge-reranker-v2-m3",
    # local_files_only=True,
)

corpus_root = Path("../assets/data/trial")
raw_corpus_r = tomllib.loads((corpus_root / "corpus_r.toml").read_text(encoding="utf-8"))["item"]
raw_corpus = tomllib.loads((corpus_root / "corpus.toml").read_text(encoding="utf-8"))["item"]
corpus = [v["text"].replace("\\", "") for v in raw_corpus_r]
queries = [v["text"].replace("[", "").replace("]", "").replace("\\", "") for v in raw_corpus]

# Find the closest sentences of the corpus for each query sentence based on cosine similarity
FIRST_PASS_TOP_K = min(20, len(corpus))
SECOND_PASS_TOP_K = min(10, FIRST_PASS_TOP_K)


def generate_qa_ranks() -> None:
    print("Generating question to answer ranks...")

    corpus_embeddings = cast(torch.Tensor, embedder.encode_document(corpus, convert_to_tensor=True))

    all_ranks: list[list[list[tuple[int, float]]]] = []

    for i, query in tqdm(enumerate(queries), total=len(queries)):
        keywords: list[str] = re.findall(r"\[(.+?)\]", raw_corpus[i]["text"])
        if not keywords:
            print(f"Warning: No keywords found for query index {i}: {query}")

        result_ranks: list[list[tuple[int, float]]] = []

        for kw in keywords:
            kw_embedding = cast(
                torch.Tensor,
                embedder.encode_query(
                    query,
                    prompt=f"为这个句子生成表示以用于检索合适的回答（关键词：{kw}）：",
                    convert_to_tensor=True,
                ),
            )
            kw_similarity_scores = embedder.similarity(kw_embedding, corpus_embeddings)[0]
            _scores, indices = torch.topk(kw_similarity_scores, k=FIRST_PASS_TOP_K)

            passeges = [corpus[idx] for idx in indices]
            ranks = reranker.rank(query + f"（关键词：{kw}）", passeges)

            result_ranks.append(
                [
                    (int(indices[int(rank["corpus_id"])]), float(rank["score"]))
                    for rank in ranks[:SECOND_PASS_TOP_K]
                ]
            )

        all_ranks.append(result_ranks)

    result_text = "\n".join(
        "".join(
            ",".join(f"{cid}:{score:.6f}" for cid, score in ranks) + "\n" for ranks in result_ranks
        )
        for result_ranks in all_ranks
    )

    (corpus_root / "ranks_qa.txt").write_text(result_text, encoding="utf-8", newline="\n")


def generate_aq_ranks() -> None:
    print("Generating answer to question ranks...")

    corpus_embeddings = cast(
        torch.Tensor, embedder.encode_document(queries, convert_to_tensor=True)
    )

    all_ranks: list[list[tuple[int, float]]] = []

    for i, query in tqdm(enumerate(corpus), total=len(corpus)):
        query_embedding = cast(torch.Tensor, embedder.encode_query(query, convert_to_tensor=True))

        similarity_scores = embedder.similarity(query_embedding, corpus_embeddings)[0]
        _scores, indices = torch.topk(similarity_scores, k=FIRST_PASS_TOP_K)

        # For answer to next question matching, there are no keywords
        passeges = [queries[idx] for idx in indices]
        ranks = reranker.rank(query, passeges)

        all_ranks.append(
            [
                (int(indices[int(rank["corpus_id"])]), float(rank["score"]))
                for rank in ranks[:SECOND_PASS_TOP_K]
            ]
        )

    result_text = "\n".join(
        ",".join(f"{cid}:{score:.6f}" for cid, score in ranks) for ranks in all_ranks
    )

    (corpus_root / "ranks_aq.txt").write_text(result_text, encoding="utf-8", newline="\n")


def main():
    generate_qa_ranks()
    generate_aq_ranks()


if __name__ == "__main__":
    main()
