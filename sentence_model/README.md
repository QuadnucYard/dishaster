# Trial System NLP Tools

This directory contains Python scripts for enhancing the trial dialogue system using Chinese NLP models.

## Features

### 1. Dialogue Ranking (`gen_ranks.py`)

Computes semantic similarity between questions and answers for intelligent response matching.

- **QA Ranks**: Maps questions to relevant answers using keywords
- **AQ Ranks**: Maps answers to follow-up questions for dialogue flow

### 2. Sentiment Analysis (`gen_sentiment_scores.py`)

Automatically computes sentiment scores for all player responses.

- Uses Chinese sentiment model: `uer/roberta-base-finetuned-jd-binary-chinese`
- Analyzes diplomatic vs confrontational toneG
- Adjusts based on response type (agreement, objection, etc.)
- Detects quality markers (apologies, solutions, dismissive language)
- Outputs scores in range [-1.0, 1.0] affecting reputation impact

### 3. Continuation Ranking (`gen_continuation_ranks.py`)

Enables multi-turn dialogue without forced alternation.

- **QQ Ranks**: Question-to-question continuations (diner can speak multiple times)
- **RR Ranks**: Response-to-response continuations (player can give multi-part replies)
- Uses BAAI/bge models for Chinese semantic understanding

## Setup

```bash
# Install dependencies (requires Python 3.13+)
cd sentence_model
uv sync
```

## Usage

### Generate All Features

```bash
uv run src/gen_all.py
```

This runs all generation steps in the correct order:

1. QA/AQ ranks (question-answer mappings)
2. Sentiment scores (response quality)
3. QQ/RR ranks (multi-turn continuations)

**Note**: The script preserves comment lines in corpus_r.toml when updating scores.

### Generate Individual Features

#### Generate All Ranks (Original)

```bash
python src/gen_ranks.py
```

Generates:

- `assets/data/trial/ranks_qa.bin` - Question to answer mappings (bincode)
- `assets/data/trial/ranks_aq.bin` - Answer to question mappings (bincode)
- `debug_output/ranks_qa.txt` - Debug text format (not committed)
- `debug_output/ranks_aq.txt` - Debug text format (not committed)

#### Generate Sentiment Scores

```bash
python src/gen_sentiment_scores.py
```

- Analyzes all responses in `corpus_r.toml`
- Updates `response_score` field in each response
- Preserves all comment lines
- Affects reputation impact in gameplay

#### Generate Continuation Ranks

```bash
python src/gen_continuation_ranks.py
```

Generates:

- `assets/data/trial/ranks_qq.bin` - Question continuations (bincode)
- `assets/data/trial/ranks_rr.bin` - Response continuations (bincode)
- `debug_output/ranks_qq.txt` - Debug text format (not committed)
- `debug_output/ranks_rr.txt` - Debug text format (not committed)

## Models Used

- **Embeddings**: `BAAI/bge-large-zh-v1.5` - Chinese text embeddings
- **Reranking**: `BAAI/bge-reranker-v2-m3` - Cross-encoder for accuracy
- **Sentiment**: `uer/roberta-base-finetuned-jd-binary-chinese` - Chinese sentiment

## Output Files

All generated files go to `../assets/data/trial/`:

- `ranks_qa.bin` - Question → Answer mappings (bincode format)
- `ranks_aq.bin` - Answer → Question mappings (bincode format)
- `ranks_qq.bin` - Question → Question continuations (bincode format) ⭐ NEW
- `ranks_rr.bin` - Response → Response continuations (bincode format) ⭐ NEW
- `corpus_r.toml` - Updated with `response_score` values ⭐ NEW

Debug text files (not committed, in `debug_output/`):

- `ranks_qa.txt`, `ranks_aq.txt`, `ranks_qq.txt`, `ranks_rr.txt` - Human-readable debug format

## How It Works

### Sentiment Scoring

1. **Base Sentiment**: Chinese RoBERTa model analyzes text tone
2. **Type Adjustment**: Different response types have different baselines
   - Agreement: +0.3 (naturally positive)
   - Question: +0.1 (neutral-positive)
   - Objection: -0.2 (can be diplomatic)
   - Perjury: -0.4 (inherently negative)
3. **Quality Markers**: Detects specific phrases
   - Positive: 抱歉, 理解, 处理, 补偿, etc.
   - Negative: 不可能, 怪, 关我什么事, etc.
4. **Final Score**: Combined and clamped to [-1.0, 1.0]

### Continuation Ranking

1. **Semantic Encoding**: Embed all statements
2. **Similarity Search**: Find related continuations
3. **Reranking**: Cross-encoder refines results with continuation context
4. **Top-K Selection**: Keep best 10 candidates per statement

## Integration with Game

The Rust code in `dishaster-data` automatically loads these files:

- Parses rank files at startup
- Uses scores for dialogue selection
- Applies sentiment to reputation calculations
- Enables multi-turn conversations

## Future Improvements

- [ ] Fine-tune sentiment model on game-specific data
- [ ] Add emotional intensity scoring
- [ ] Implement dialogue act classification
- [ ] Create interactive labeling tool
- [ ] Support for more languages
