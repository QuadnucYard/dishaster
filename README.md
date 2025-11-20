# 🍽️ Dishaster

![game-icon](./godot/icon.svg)

> _嚼得菜根，做得大事_ — Chew on humble roots, achieve great deeds.

**Dishaster** is a data-driven university canteen management simulation where hundreds of autonomous diners with unique personalities, preferences, and memories create emergent behavioral patterns. Built with Rust and Godot, it combines strategic planning with the satisfaction of observing complex systems unfold.

## Core Features

### Agent-Based Simulation

- **Autonomous diners**: Each with persistent identity, personality traits (frugality, patience, adventurousness), and memory systems
- **Emergent Behavior**: Individual decisions compound into complex crowd dynamics—no scripted events
- **Memory & Learning**: Diners remember dish quality, service experiences, and develop lasting preferences

### Strategic Gameplay

- **Dual-Phase Loop**: Preparation (configure prices) → Service (observe real-time simulation) → Settlement (review metrics)
- **Roguelite Progression**: Daily management decisions with permanent upgrades and random incidents
- **Dynamic Systems**: Reputation tracking, food safety risk index, and probabilistic feedback

### Dialogue System

- **NLP-Powered Trials**: Manosaba inspired conversations using semantic matching
- **Complete Randomness**: Every trial experience is unique—same keywords yield different response options

### Technical Excellence

- **Deterministic Simulation**: Seeded RNG ensures reproducibility for debugging and testing
- **Engine-Agnostic Core**: Pure Rust simulation with clean CQRS interface
- **Data-Driven Design**: 90+ dishes, multiple layouts, 400+ dialogue entries—all in RON configs

## Architecture

Dishaster employs a strict layered architecture with unidirectional data flow:

```txt
┌─────────────────────────────────────────────┐
│          Godot Presentation Layer           │
│  (dishaster-godot, dishaster-godot-ui)      │
└─────────────────────────────────────────────┘
                     ↓ Events ↑ Commands
┌─────────────────────────────────────────────┐
│         Interface Layer (CQRS)              │
│  Commands · Queries · Events · Responses    │
│         (dishaster-interface)               │
└─────────────────────────────────────────────┘
                     ↓ ↑
┌─────────────────────────────────────────────┐
│          Simulation Core (ECS)              │
│  Bevy ECS @ any TPS · Agent Systems         │
│  Navigation · Queues · Trials · Feedback    │
│         (dishaster-core)                    │
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│         Data Models & Registry              │
│  (dishaster-models, dishaster-data)         │
└─────────────────────────────────────────────┘
```

## Getting Started

### Prerequisites

- **Rust** 1.93+ (uses nightly features)
- **Godot** 4.5
- **Nushell** (for build scripts)

### Build & Run

```nu
# Build the project
just build

# Run in development mode (Godot editor)
just dev

# Run tests
just test

# Format code
just fmt

# Export builds
just export <platform>  # debug/release

# Package for distribution
just pack <platform>
```

## Project Structure

The repository is structured as a Rust workspace and the Godot project. Crates are split into two categories: `dishaster-*` (game-specific crates) and `dishrupt-*` (reusable framework crates).

Top-level layout:

```text
dishaster/
├── crates/
│   ├── dishaster-*     # Game-specific crates (core, UI, data, glue)
│   └── dishrupt-*      # Reusable framework crates (utils, ECS, godot helpers)
├── assets/             # Game content files, RON configs
│   └── data/           # dishes.ron, canteens.ron, mgmt_decisions.ron, trial/
├── godot/              # Godot project, .tscn scenes, export presets
├── sentence_model/     # Optional Python helpers for NLP/rank generation (model build)
├── design/             # Game design docs (Chinese)
└── justfile            # Build automation (Nu shell)
```

For a full list of crates and responsibilities, see `crates/README.md`.

## License

See [LICENSE](LICENSE) for details.
