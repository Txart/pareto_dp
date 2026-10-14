# Publishing a new version

1. Bump the version in `Cargo.toml` (e.g., `0.1.2` → `0.1.3`)
2. Commit the change
3. Create a git tag matching the version
4. Push the commit and the tag

```bash
git add Cargo.toml
git commit -m "bump version to x.x.x"
git tag x.x.x
git push origin main --tags
```

The GitHub Actions CI workflow triggers on tag pushes and will:
- Build wheels for all platforms (Linux, macOS, Windows, musllinux)
- Publish to PyPI automatically
