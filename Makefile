all: fmt build test

build:
	cargo build

test:
	cargo test

fmt:
	find . -type f -iname "*.rs" -exec rustfmt \{\} \;

watch:
	nodemon -e rs -x "cargo test --jobs 1 -- --test-threads=1"

wc:
	find . -type f -iname "*.rs" -exec wc -l \{\} \; | sort -n

coverage:
	docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin -o Html

clean:
	cargo clean
