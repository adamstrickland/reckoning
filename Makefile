.DEFAULT_GOAL := build

test:
	cargo test

test-watch:
	@echo
	@make test
	@echo
	@echo "---"
	@echo

watch:
	@echo "Watching..."
	@fswatch -or --event=OwnerModified $(NAME) src tests | xargs -n1 -I{} make test-watch

build: test
	cargo build
