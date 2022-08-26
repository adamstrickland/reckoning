test-watch:
	@echo
	cargo test
	@echo
	@echo "---"
	@echo

watch:
	@echo "Watching..."
	@fswatch -or --event=OwnerModified $(NAME) src tests | xargs -n1 -I{} make test-watch

