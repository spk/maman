all: build test ## Build and test

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Build
	@cargo build

test: ## Test
	cargo test

bench: ## Bench
	cargo bench

docs: build
	@cargo doc --no-deps

.PHONY: all build test bench docs help
