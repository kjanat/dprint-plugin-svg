# https://just.systems

alias f := fmt
alias l := lint
alias t := test
alias bw := build-wasm
alias s := schema
alias cmt := commit

default:
    just --list --unsorted

fmt:
    dprint fmt

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features

build-wasm:
    cargo build --release --target wasm32-unknown-unknown

schema:
    cargo run --features schema --bin generate-schema -- deployment/schema.json
    dprint fmt --log-level error

plugin-path:
    @if [ ! -f target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm ]; then just build-wasm; fi
    @if [ ! -f target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    @echo "$(pwd)/target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm"

# Let gippity write a nice commit message
[arg("model", long="model", short="m")]
[arg("variant", long="variant", short="v")]
commit model="openai/gpt-5.4" variant="medium" *$MESSAGE:
    opencode run --command commit --model={{ model }} --variant={{ variant }} "$MESSAGE"
