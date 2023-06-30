
.PHONY: test
test: fmt
	@cargo nextest run

.PHONY: testall
testall: fmt
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

# 代码覆盖率: 代码覆盖率是一种度量代码保护程度的指标，一般而言，覆盖率越高代表着代码越值得信赖
.PHONY: coverage
coverage:
	@cargo tarpaulin --out Html
