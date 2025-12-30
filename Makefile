all: fmt build test

build:
	cargo build

test:
	cargo test

fmt:
	cargo fmt

watch:
	nodemon -e rs -x "cargo test"

# identify the largest source files
wc:
	find . -type f -iname "*.rs" -exec wc -l \{\} \; | sort -n

coverage:
	docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin -o Html

clean:
	cargo clean

# prints the size of the types
#
# perl -e 'print sort { length($b) <=> length($a) } <>' sorts by line length in descending order
# escaping $ in Makefile by doubling it
print-type-size:
	cargo clean && RUSTFLAGS="-Zprint-type-sizes" cargo build -p rusty_parser | perl -e 'print sort { length($$b) <=> length($$a) } <>'
