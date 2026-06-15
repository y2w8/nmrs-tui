build mode="":
  cargo build --{{mode}}

run:
  cargo run

debug:
  NMRS_LOG=trace cargo run

check:
  cargo check

package:
  cargo package

publish:
  cargo publish

release level="patch":
  cargo release {{level}} --execute

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings

ci: fmt lint
