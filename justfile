set shell := ["nu", "-c"]

alias b := build

dev:
    RUST_BACKTRACE=full godot godot/project.godot

fmt:
    cargo fmt

build:
    cargo build

test:
    cargo test --tests --workspace

export *ARGS:
    nu scripts/export.nu {{ARGS}}

pack *ARGS:
    nu scripts/pack.nu {{ARGS}}
