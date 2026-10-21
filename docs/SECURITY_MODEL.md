# Rust Lab security model

Status: v0.4.2-alpha.2 public alpha candidate

## Purpose

Rust Lab allows a local user to execute graphs and experiment with Rust projects while reducing the
chance that untrusted input reaches unrelated host files, credentials, networks, or processes. The
security controls are defense in depth; they do not turn Rust Lab into a public multi-tenant code
execution service.

## Trust boundaries

The following inputs are untrusted:

- imported GitHub repositories and Git references;
- crates and build scripts downloaded from crates.io;
- graph/project files opened by the user;
- generated or model-produced code and commands;
- HTTP responses consumed by graph nodes;
- file names, Cargo output, and compiler diagnostics shown in the UI.

The local user, Rust Lab installation, configured execution policy, and supported sandbox runtime
are inside the intended trust boundary. The host Docker daemon is highly privileged and is never
made available to the Compose container.

The desktop backend binds a fresh loopback port for every application instance. The Tauri command
channel delivers a 256-bit random API token to that instance's UI; API requests also require the
`X-Requested-With: Polyglot-Lab` marker. The browser-hosted build receives the token only in its
same-origin HTML response. CORS remains defense in depth and is not treated as authentication.

## Graph execution controls

- Graphs are structurally and semantically validated before execution.
- File reads and writes are limited by host-controlled allowlists and canonical path checks.
- Graph content cannot relax host policy.
- HTTP access is restricted to configured domains.
- Requests and execution have size and time limits.
- Generated-project compile checks use offline Cargo.

## Workspace controls

- Project IDs and paths are validated; symlinks and internal `.git`, `.cargo`, `target`, and
  `.rust-lab` paths are excluded from editor access.
- Source editing is limited to an explicit text-file allowlist and a 512 KiB per-file limit.
- Public repository imports are shallow, non-interactive HTTPS clones with size/file limits.
- Every dependency modification and Cargo command requires an explicit UI confirmation.
- Offline Rust, Go, Python, and C++ execution prefers Bubblewrap with namespace isolation and falls
  back to Docker. Dependency Fetch is Docker-only because it requires network access.
- Sandbox selection is fail-closed. If Bubblewrap cannot start and the required Docker image is not
  already present, the API returns `503 Service Unavailable`; project code is never run directly on
  the host.
- Docker execution drops all capabilities, enables `no-new-privileges`, limits CPU, memory, PIDs,
  and time, uses a read-only root filesystem, and disables the network for execution.
- Docker is configured with `--pull never`; commands cannot silently download a runner image. Rust
  uses `rust:bookworm`. Go, Python, and C++ use digest-pinned `linux/amd64` images documented in
  `docs/TROUBLESHOOTING.md`; alternative administrator-provided images require explicit environment
  configuration.
- Fetch is separated from execution. It has network access but uses a project-specific Cargo home,
  disables Cargo credential providers and external Git transport, and receives no host credentials.
- Each Cargo result is appended to a local trajectory record.

## Availability reporting

`GET /api/lab/capabilities` probes the supported backends. The UI disables Cargo actions when no
safe backend is ready and separately reports when networked dependency Fetch is unavailable.
Presence of an executable alone is not presented as readiness.

## Known non-goals

- Protecting a host from a malicious local administrator.
- Safely serving mutually untrusted users over the internet.
- Guaranteeing that all third-party crates are free of vulnerabilities or malicious behavior.
- Protecting data intentionally pasted into an edited project or sent to an allowlisted endpoint.
- Supporting host Docker-socket mounting from the standard container deployment.

## Future security work

- Add project archive import with the same limits and path validation.
- Produce signed release artifacts and a software bill of materials.
- Add automated dependency policy and vulnerability review without making release builds depend on
  a live advisory service.
- Add signed first-party multi-architecture polyglot sandbox images.
