cargo-doc-command := "cargo doc --no-deps --lib --all-features --document-private-items"

check:
  cargo check

build:
  cargo clean --quiet -r
  cargo build --release --quiet

doc:
  cargo clean --doc --quiet
  {{cargo-doc-command}}

doc-open:
  rm -r target/doc
  {{cargo-doc-command}} --open

example api_key access_token:
  KITE_API_KEY={{api_key}} KITE_ACCESS_TOKEN={{access_token}} cargo run --example sample
