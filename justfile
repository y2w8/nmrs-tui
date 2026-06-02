build mode="":
  cargo build --{{mode}}

run:
  cargo run

debug:
  RUST_LOG=debug cargo run

deploy:
  cargo build --release
