PREFIX ?= /usr/local

BUILD = target/release/maman
MANPAGE = man/man1/maman.1

all: build install ## Build and install

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

$(BUILD): ## Build
	@which cargo > /dev/null || { echo "https://www.rust-lang.org/"; exit 1; }
	@cargo build --release

build: $(BUILD)

INSTALL = $(PREFIX)/bin/maman

$(INSTALL): ## Install
	install -dm755 $(PREFIX)/bin/ $(PREFIX)/share/man/man1/
	install -sm755 $(BUILD) $(PREFIX)/bin/
	install -m644 $(MANPAGE) $(PREFIX)/share/man/man1/

install: build $(INSTALL)

clean: ## Clean
	rm -rf $(BUILD)

uninstall: ## Uninstall
	rm $(PREFIX)/bin/maman $(PREFIX)/share/$(MANPAGE)

manpage: ## Generate manpage
	@which asciidoctor > /dev/null || { echo "install asciidoctor"; exit 1; }
	@find doc/ -type f -exec asciidoctor -b manpage -D man/man1 {} \;

.PHONY: all install clean uninstall help
