set windows-shell := ["nu", "-c"]

alias b := build

dev $RUST_BACKTRACE="full":
    godot godot/project.godot

fmt:
    cargo fmt

build *ARGS:
    cargo build {{ARGS}}

build-web $RUSTFLAGS="
        -C link-args=-sSIDE_MODULE=2
        -Z link-native-libraries=no
        -C llvm-args=-enable-emscripten-cxx-exceptions=0":
    cargo build -p dishaster-godot-ext --features web -Zbuild-std --target wasm32-unknown-emscripten

build-web-release $RUSTFLAGS="
        -C link-args=-sSIDE_MODULE=2
        -Z link-native-libraries=no
        -C llvm-args=-enable-emscripten-cxx-exceptions=0":
    cargo build --release -p dishaster-godot-ext --features web -Zbuild-std --target wasm32-unknown-emscripten

test:
    cargo test --tests --workspace

export *ARGS:
    nu scripts/export.nu {{ARGS}}

pack *ARGS:
    nu scripts/pack.nu {{ARGS}}
