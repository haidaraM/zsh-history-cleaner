default:
	@echo "Available commands:"
	@echo "  make update-changelog-for-next-release  - Update CHANGELOG.md for the next release"
	@echo "  make bump-version                      - Bump the version and update CHANGELOG.md"

bump-version:
	@echo "Bumping version..."
	@cargo release version $(shell git-cliff --bumped-version) --execute
	@cargo release hook --execute --no-confirm

update-changelog-for-next-release:
	@echo "Updating CHANGELOG.md for next release..."
	@git-cliff -o CHANGELOG.md
	@git add CHANGELOG.md
	@git commit -m "doc(release): update CHANGELOG.md for next release"
