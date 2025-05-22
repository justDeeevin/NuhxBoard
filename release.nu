cargo publish --no-verify -Z package-workspace --workspace --exclude iced-multi-window --dry-run
git tag -s -m "" $"v(cat ./Cargo.toml | from toml | get workspace.package.version)"
git push --tags
