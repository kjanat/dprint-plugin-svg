# https://just.systems

alias f := fmt
alias l := lint
alias t := test
alias bw := build-wasm
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
    CFLAGS_wasm32_unknown_unknown='-DNDEBUG -Disalpha(c)=(((c)>=65&&(c)<=90)||((c)>=97&&(c)<=122)) -Disdigit(c)=((c)>=48&&(c)<=57)' cargo build --release --target wasm32-unknown-unknown

plugin-path:
    @if [ ! -f target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm ]; then just build-wasm; fi
    @if [ ! -f target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm ]; then echo "Plugin artifact not found after build." >&2; exit 1; fi
    @echo "$(pwd)/target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm"

# Let gippity write a nice commit message
[arg("model", long="model", short="m")]
[arg("variant", long="variant", short="v")]
commit model="openai/gpt-5.4" variant="medium" *$MESSAGE:
    opencode run --command commit --model={{ model }} --variant={{ variant }} "$MESSAGE"
