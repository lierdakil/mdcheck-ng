options.md: module.nix flake.nix
	rm -f ./options.md || true
	cp $$(nix build .#doc --no-link --print-out-paths) ./options.md
	chmod 644 ./options.md
	git add ./options.md
	git commit -m 'Update options.md'
