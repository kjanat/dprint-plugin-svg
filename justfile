# https://just.systems

set positional-arguments := true

alias f := fmt
alias l := lint
alias t := test
alias bw := build-wasm
alias s := schema
alias b := book
alias cmt := commit

target := "wasm32-unknown-unknown"
wasmpath := "target" / target / "release/dprint_plugin_svg.wasm"
schemapath := "deployment/schema.json"

default:
    just --list --unsorted

fmt:
    @if [ ! -f '{{ wasmpath }}' ]; then just build-wasm; fi
    @if [ ! -f '{{ wasmpath }}' ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    dprint fmt

lint:
    cargo clippy --all-targets --all-features -- -D warnings -D clippy::all

test:
    cargo test --all-targets --all-features

build-wasm:
    cargo build --release --target {{ target }}

schema:
    cargo run --features schema --bin generate-schema -- {{ schemapath }}
    dprint fmt --log-level error {{ schemapath }}

book:
    mdbook build docs

plugin-path:
    @if [ ! -f '{{ wasmpath }}' ]; then just build-wasm; fi
    @if [ ! -f '{{ wasmpath }}' ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    @echo "$(pwd)/{{ wasmpath }}"

# Let gippity write a nice commit message
[arg("model", long="model", short="m")]
[arg("variant", long="variant", short="v")]
commit model="openai/gpt-5.4" variant="medium" *MESSAGE:
    opencode run --command commit --model={{ model }} --variant={{ variant }} "$@"
