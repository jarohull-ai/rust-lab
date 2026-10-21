# Troubleshooting

## API shows Disconnected

Start Rust Lab from the repository root with `./scripts/run-local.sh`. Confirm that another process
is not already using the selected port, then open the exact address printed by the server. The
default is <http://127.0.0.1:3030>.

Check the API directly:

```bash
curl -fsS http://127.0.0.1:3030/api/health
```

All other API routes require the per-launch token printed by the standalone API process. The
desktop and same-origin browser UI receive this token automatically; do not paste it into bug
reports or commit it to Git.

## Cargo actions are disabled

The Lab deliberately disables commands until a supported sandbox passes a live readiness probe.
Inspect the status card in the Cargo panel or query:

```bash
curl -fsS \
  -H 'X-Requested-With: Polyglot-Lab' \
  -H "X-Lab-Token: $RUST_LAB_API_TOKEN" \
  http://127.0.0.1:3030/api/lab/capabilities
```

On Linux, install Bubblewrap for offline commands. To enable dependency Fetch and the Docker
fallback, start Docker and provision the runner image explicitly:

```bash
docker pull rust:bookworm
docker image inspect rust:bookworm
docker pull 'golang:1.25-bookworm@sha256:c268a04d59aea0b180ed9946a658cfab9e7b3391dc90eed6e4969ccff98c851f'
docker pull 'python:3.12-slim-bookworm@sha256:2ed6491b93cd49272ee6de2b5a38440c3448360322c089fc23e370722d74179d'
docker pull 'dockcross/linux-x64@sha256:552d7f84b9978bd9cbd8a1b1e2ad40963ff135750b2c21311686b1b31bafd184'
```

Use the refresh button in the Cargo status card after changing the environment.

## Fetch is disabled but Check is available

This is expected when Bubblewrap is ready but Docker is not. Offline Check/Test/Run can use already
cached dependencies. Networked Fetch requires Docker because it runs in a separate contained phase.

## A dependency is missing during offline Check

Run **Fetch** once while online. Each Lab project has its own Cargo cache, so fetching in one project
does not populate another project's cache.

## Lab commands are unavailable in Compose

The standard Compose deployment intentionally has no Docker socket and no elevated namespace
privileges. Graph mode remains available, while real Cargo execution is disabled. Run Rust Lab
directly on a supported Linux host for full Lab mode. Do not mount the host Docker socket as a
workaround.

## UI dependencies fail to install

Confirm Node.js 22+ and npm 10+, then remove only the generated `ui/node_modules/` directory and run
`npm --prefix ui ci` again. Do not delete `ui/package-lock.json`.

## Preparing a bug report

Run `./scripts/verify.sh` and include the failing command, Rust Lab version, operating system, Rust
and Node.js versions, and `/api/lab/capabilities` response. Remove credentials, personal paths,
private source code, and raw project payloads.
