set shell := ["nu", "-c"]

alias b := build

dev:
    RUST_BACKTRACE=1 godot godot/project.godot

fmt:
    cargo fmt

build:
    cargo build

export *ARGS:
    nu scripts/export.nu {{ARGS}}

pack *ARGS:
    nu scripts/pack.nu {{ARGS}}
