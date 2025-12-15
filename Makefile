options.md: module.nix flake.nix
	rm -f ./options.md || true
	cp --no-preserve=all $$(nix build .#doc --no-link --print-out-paths) ./options.md
	git add ./options.md
	git commit -m 'Update options.md'

systemd: module.nix flake.nix
	rm -rf ./systemd/* || true
	cp --no-preserve=all $$(nix build .#units --no-link --print-out-paths)/* ./systemd/
	sed -i 's!/nix/store/.*-mdcheck-ng-.*/bin!/path/to!' ./systemd/mdcheck-ng.service
	sed -i 's!/nix/store/.*-mdcheck-ng\.toml!/path/to/mdcheck-ng.toml!' ./systemd/mdcheck-ng.service
	git add ./systemd/
	git commit -m 'Update ./systemd'
