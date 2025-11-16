# Trial Snapshot Tests

Snapshot tests for trial sessions using [insta](https://insta.rs/). Each test runs a randomized trial session with a fixed seed to verify dialogue flow, response selection, impact calculations, and continuation logic.

## What's Tested

- **Dialogue generation**: Diner complaints and manager responses with emotions/gestures
- **Response selection**: Keyword-based matching via QA/AQ/QQ/RR ranks
- **Impact calculation**: Reputation and psychology (mood/trust/patience) changes
- **Continuation logic**: Multi-turn conversations based on scores
- **Topic filtering**: Optional topic-specific speech filtering

## Test Approach

Tests use **production corpus** loaded from `assets/data/`. Each test:
1. Starts a session with fixed seed and optional topic filter
2. Randomly simulates player responses (or skip responding)
3. Captures complete session state in YAML snapshots
4. Verifies output is deterministic for the given seed

All randomness is controlled by seed - tests verify that identical seeds produce identical results.

## Running Tests

```bash
cargo test -p dishaster-trial        # Run all tests
cargo insta review                    # Review snapshot changes
cargo insta accept                    # Accept new snapshots
```
