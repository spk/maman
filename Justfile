default_prefix := "/usr/local"
default_manpage_path := "share/man/man1/"
default_manpage := "maman.1"

all: build test

@build:
    cargo build

@test:
    cargo test --all -- --quiet

@bench:
    cargo bench

@docs: build
    cargo doc --no-deps

@format:
    cargo fmt --all -- --check

@lint:
    cargo clippy -- -D warnings

@install:
    cargo build --release
    find doc/ -type f -exec asciidoctor -b manpage -D man/man1 {} \;
    install -dm755 {{env_var_or_default("PREFIX", default_prefix)}}/bin/
    install -dm755 {{env_var_or_default("PREFIX", default_prefix)}}/{{default_manpage_path}}
    install -sm755 target/release/maman {{env_var_or_default("PREFIX", default_prefix)}}/bin/
    install -m644 man/man1/{{default_manpage}} {{env_var_or_default("PREFIX", default_prefix)}}/{{default_manpage_path}}

@uninstall:
    rm -f {{env_var_or_default("PREFIX", default_prefix)}}/bin/maman
    rm -f {{env_var_or_default("PREFIX", default_prefix)}}/{{default_manpage_path}}{{default_manpage}}
