set tag=%TRAVIS_TAG%
if defined tag (
  echo Skip build
) else (
  cargo build --target %TARGET% && \
  cargo build --target %TARGET% --release && \
  cargo test --target %TARGET% && \
  cargo test --target %TARGET% --release && \
  cargo run --target %TARGET% --bin uvm -- --help && \
  cargo run --target %TARGET% --bin uvm --release -- --help
)
