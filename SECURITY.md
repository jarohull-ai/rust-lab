# Security policy

## Supported versions

Rust Lab is currently a public alpha. Security fixes are applied to the latest commit on `main`
and to the most recent GitHub pre-release. Older development snapshots are not supported.

## Reporting a vulnerability

Please do **not** open a public issue for a suspected vulnerability.

Use GitHub's private vulnerability reporting for this repository:

1. Open the repository's **Security** tab.
2. Choose **Advisories** and **Report a vulnerability**.
3. Include the affected version or commit, host platform, reproduction steps, impact, and any
   suggested mitigation.

If private reporting is temporarily unavailable, open a minimal issue asking the maintainer to
enable a private security contact. Do not include exploit details, tokens, personal data, or a
working proof of concept in that issue.

You should receive an acknowledgement within seven days. The maintainer will coordinate validation,
remediation, release timing, and disclosure with the reporter.

## Deployment warning

Rust Lab v0.3.0 is designed as a local, single-user application. It executes untrusted Rust only
through its documented sandbox paths, but it is **not** a multi-tenant remote code execution
service. Keep the default loopback binding. Do not expose it to the public internet.

Do not mount the host Docker socket into the Rust Lab Compose container. Access to that socket is
effectively host-level control and is outside the supported security model.

See [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) for trust boundaries, controls, and known
non-goals.
