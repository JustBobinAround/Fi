test:
	cargo build
	sh test.sh
	echo ""
	ls --color=always | ./target/debug/fi
