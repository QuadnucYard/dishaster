set shell := ["nu", "-c"]

alias b := build

dev $RUST_BACKTRACE="full":
    godot godot/project.godot

fmt:
    cargo fmt

build *ARGS:
    cargo build {{ARGS}}

build-web $RUSTFLAGS="
        -C link-args=-sSIDE_MODULE=2
        -Zlink-native-libraries=no
        -Cllvm-args=-enable-emscripten-cxx-exceptions=0" *ARGS:
    cargo +nightly build -p dishaster-godot-ext --features web -Zbuild-std --target wasm32-unknown-emscripten {{ARGS}}

test:
    cargo test --tests --workspace

export *ARGS:
    nu scripts/export.nu {{ARGS}}

pack *ARGS:
    nu scripts/pack.nu {{ARGS}}
