all: build test ## Build and test

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Build
	@cargo build

test: ## Test
	@RUST_TEST_THREADS=1 cargo test

bench: ## Bench
	@RUST_TEST_THREADS=1 cargo bench

docs: build
	@cargo doc --no-deps

.PHONY: all build test bench docs help
