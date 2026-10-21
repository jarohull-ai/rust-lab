# Release process

Rust Lab releases are cut from `polyglot-lab`. Versions below 1.0 may change project and API formats, but
must still provide clear migration notes.

## Candidate gate

1. Update versions in the workspace and UI package. Keep `src-tauri/tauri.conf.json` at the numeric
   `MAJOR.MINOR.PATCH` product version: MSI and RPM reject textual prerelease identifiers. The Git
   tag and GitHub prerelease carry the `-alpha.N` suffix.
2. Move relevant changelog entries from `Unreleased` into a dated version section.
3. Run `./scripts/verify.sh` from a clean checkout.
4. Run a local smoke test for Graph execution, Cargo export, project creation, sandbox capability
   reporting, dependency Fetch, and offline Cargo Check.
5. Confirm the README support matrix and security limitations still match behavior.
6. Confirm CI succeeds on the release commit and no generated workspaces, traces, tokens, or raw
   payloads are tracked.

## Publishing

1. Merge the reviewed candidate into `polyglot-lab`.
2. Create an annotated `vX.Y.Z` tag on the verified commit.
3. Push the tag. The release workflow creates one prerelease first, then passes its release ID to
   every platform job so concurrent builds only upload their own assets.
4. Mark 0.x releases as pre-releases while the public alpha/beta warning applies.
5. Include user-visible changes, known limitations, verification results, and upgrade notes.

The internal workspace crates are `publish = false`. Publishing any crate to crates.io is a
separate product decision and requires unique package names, crate-level metadata, API review, and
an explicit release gate.
