# https://just.systems

set unstable := true
set positional-arguments := true

alias f := fmt
alias l := lint
alias t := test
alias bw := build-wasm
alias s := schema
alias b := book
alias cmt := commit

target := "wasm32-unknown-unknown"
wasmpath := "target" / target / "wasm-release/dprint_plugin_svg.wasm"
schemapath := "deployment/schema.json"

[private]
default:
    just --list --unsorted

[group('check')]
fmt:
    @if [ ! -f '{{ wasmpath }}' ]; then just build-wasm; fi
    @if [ ! -f '{{ wasmpath }}' ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    dprint fmt

[group('check')]
lint:
    cargo clippy --all-targets --all-features -- -D warnings -D clippy::all

[group('check')]
test:
    cargo test --all-targets --all-features

[group('build')]
build-wasm:
    cargo build --profile wasm-release --target {{ target }}

[group('build')]
schema:
    cargo run --features schema --bin generate-schema -- {{ schemapath }}
    dprint fmt --log-level error {{ schemapath }}

[group('build')]
[group('docs')]
book:
    mdbook build docs

[group('docs')]
plugin-path:
    @if [ ! -f '{{ wasmpath }}' ]; then just build-wasm; fi
    @if [ ! -f '{{ wasmpath }}' ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    @echo "$(pwd)/{{ wasmpath }}"

# Let gippity write a nice commit message
[arg("model", long="model", short="m")]
[arg("variant", long="variant", short="v")]
[group('docs')]
commit model="openai/gpt-5.4" variant="medium" *MESSAGE:
    opencode run --command commit --model={{ model }} --variant={{ variant }} "$@"
