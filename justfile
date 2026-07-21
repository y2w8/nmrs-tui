build mode="":
  cargo build --{{mode}}

run:
  cargo run

debug $argument="":
  @NMRS_LOG=trace cargo run -- $argument
  @bat ~/.cache/nmrs-tui/nmrs-tui.log

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
