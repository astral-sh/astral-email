# Contributing

## Releases

Releases can only be performed by Astral team members.

### Configuration

Release gates and tag protections are managed in
[github-policies](https://github.com/astral-sh/github-policies). Before the first
release, run **Apply** in [crates-policies](https://github.com/astral-sh/crates-policies)
to bootstrap the crate and enable Trusted Publishing for `release.yml` in the
`release` environment.

The workflows use these environment secrets:

| Environment   | Secret           | Value                                                     |
| ------------- | ---------------- | --------------------------------------------------------- |
| `automations` | `STS_API_URL`    | Astral's automation-broker base URL, without `/exchange`. |
| `automations` | `OPENAI_API_KEY` | An API key for the Codex changelog rewrite.               |
| `release`     | `STS_API_URL`    | Astral's release-broker base URL, without `/exchange`.    |

### Prepare and publish

1. Run **Prepare release** from `main`. Leave `version` empty to use `Cargo.toml`
   for the first release, then automatically bump the minor version for `breaking`
   changes or the patch version otherwise, excluding internal changes. To override,
   provide an exact stable Cargo version without a leading `v`.
2. Review and merge the generated version and changelog PR.
3. Run **Release** from `main` with that version. **Dry-run** validates the package
   and release notes without publishing.
4. Have another Astral team member approve `release-gate` to publish the crate and
   create the GitHub release. Retries skip crate uploads already on crates.io.
