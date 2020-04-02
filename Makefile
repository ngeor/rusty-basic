all: fmt build test

build:
	cargo build

test:
	cargo test

fmt:
	find . -type f -iname "*.rs" -exec rustfmt \{\} \;

watch:
	nodemon -e rs -x "cargo test"

wc:
	find . -type f -iname "*.rs" -exec wc -l \{\} \; | sort -n
