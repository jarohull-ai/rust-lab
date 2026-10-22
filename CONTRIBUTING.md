# Contributing to Rust Lab

Thanks for helping make Rust learning more approachable without hiding what Rust actually does.

## Before you start

- Search existing issues and pull requests.
- Use an issue for behavior changes or substantial UI proposals before writing a large patch.
- Keep changes focused on one complete user-facing slice.
- Do not weaken sandboxing, explicit approval, path validation, execution policy, or trajectory
  recording.
- Treat imported projects and generated/model-produced content as untrusted.

## Local setup

Requirements are listed in the [README](README.md#requirements). Then run:

```bash
git clone https://github.com/jarohull-ai/rust-lab.git
cd rust-lab
npm --prefix ui ci
./scripts/verify.sh
```

Start the product with `./scripts/run-local.sh`.

## Change requirements

1. Keep English as the canonical UI language and update the Polish translation in the same change.
2. Add or update tests for changed behavior.
3. Update user documentation and `CHANGELOG.md` when behavior is externally visible.
4. Run `./scripts/verify.sh` before opening a pull request.
5. Keep runtime traces, credentials, imported repositories, Cargo caches, and raw payloads out of
   Git.
6. Use Conventional Commit-style subjects where practical, for example
   `feat(lab): report sandbox readiness`.

## Pull requests

Explain the problem, the user-visible result, the safety impact, and the verification performed.
Small screenshots are welcome for UI changes. A maintainer may request that a security-sensitive
change be split from unrelated refactoring.

By contributing, you agree that your contribution is licensed under the project's
`MIT OR Apache-2.0` terms.

Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
