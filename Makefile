.PHONY: release

# Bumps the patch version, tags and pushes.
# The v* tag triggers .github/workflows/docker-deploy.yml, which builds and
# pushes the image to ghcr.io.
release:
	@git diff --quiet HEAD || { echo "working tree dirty"; exit 1; }
	@v=$$(sed -n '1,/^version = /s/^version = "\(.*\)"/\1/p' Cargo.toml); \
	new=$$(echo $$v | awk -F. '{print $$1"."$$2"."$$3+1}'); \
	sed '1,/^version = /s/^version = ".*"/version = "'$$new'"/' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml; \
	cargo update -p dyn-ip; \
	git commit -qam "Release v$$new" && git tag "v$$new" && git push -q origin HEAD "v$$new"; \
	echo "pushed v$$new — GitHub Actions is building the image"
