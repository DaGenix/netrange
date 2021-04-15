# netrange

- [ ] Bump the version in [Cargo.toml](./Cargo.toml).
- [ ] Update [CHANGELOG.md](./CHANGELOG.md) with the new version and date.
- [ ] Create a commit and open a new PR.
- [ ] Once tests complete, create a pre-release against the PR branch
  with a new tag of the form "netrange-1.2.3" where "1.2.3" is the version
  of netrange. Make sure the release notes contains the text from CHANGELOG.md.
- [ ] Wait for Pre-compiled binaries to be uploaded to the release.
- [ ] Update the release to no longer mark it as a pre-release.
- [ ] Use `cargo publish` to upload the package to crates.io
- [ ] Merge the PR with the comment "bors r+".

# libnetrangemerge

- [ ] Bump the version in [Cargo.toml](./libnetrangemerge/Cargo.toml).
- [ ] Update [CHANGELOG.md](./libnetrangemerge/CHANGELOG.md) with the new
  version and date.
- [ ] Update netrange's [Cargo.toml](./Cargo.toml) to use the new version of
  libnetrangemerge.
- [ ] Create a commit and open a new PR.
- [ ] Once tests complete, create a new tag against the PR branch of the form
  "libnetrangemerge-4.5.6" where "4.5.6" is the version of libnetrangemerge.
- [ ] Use `cargo publish` to upload the package to crates.io
- [ ] Merge the PR with the comment "bors r+".
- [ ] Consider if a new version of netrange should also be released.
