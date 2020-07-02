SHELL=/usr/bin/env bash

.PHONY: execute execute_delay print_png print_dot print test build pdf cargo_help help

execute: ## Execute a randomly generated DAG
	cargo run -- -m execute -x 50 -n 40 -p 40 -d

execute_delay:  ## Execute a smaller DAG with two-second delays at each level
	cargo run -- -m execute -x 8 -n 5 -p 40 -d --delay

print_png:   ## Print random DAG in png format using dot
	cargo run -- -m print | dot -Tpng -o dag.png

print_dot:   ## Print default random DAG in dot format
	cargo run -- -m print | dot

print:       ## Print random DAG in dot format with command-line flags
	cargo run -- -m print -x 20 -n 15 -p 20 -d | dot

test:  ## Run all unit tests, takes roughly 30 seconds total
	cargo test

build:  ## Build executable
	cargo build

pdf: ## Make README into a pdf
	pandoc README.md -s -o README.pdf

cargo_help:  ## Show executable help
	cargo run -- --help

help:          ## Show this help
	@IFS=$$'\n' ; \
	help_lines=(`fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##/:/'`); \
	printf "\nRun \`make\` with any of the targets below to reach the desired target state.\n\n" ; \
	printf "%-30s %s\n" "target" "help" ; \
	printf "%-30s %s\n" "------" "----" ; \
	for help_line in $${help_lines[@]}; do \
		IFS=$$':' ; \
		help_split=($$help_line) ; \
		help_command=`echo $${help_split[0]} | sed -e 's/^ *//' -e 's/ *$$//'` ; \
		help_info=`echo $${help_split[2]} | sed -e 's/^ *//' -e 's/ *$$//'` ; \
		printf '\033[36m'; \
		printf "%-30s %s" $$help_command ; \
		printf '\033[0m'; \
		printf "%s\n" $$help_info; \
	done
