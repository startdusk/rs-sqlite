
.PHONY: test
test: fmt
	@cargo nextest run
	@bundle exec rspec

.PHONY: fmt
fmt:
	@cargo fmt 
	@cargo fmt -- --check
	@cargo clippy --all-targets --all-features --tests --benches -- -D warnings

.PHONY: codeline
codeline:
	@tokei .
