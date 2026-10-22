# Roadmap

Rust Lab is one product with two complementary modes: visual learning and real Cargo experiments.
The roadmap intentionally avoids splitting it into multiple applications.

## v0.3 — Public alpha

- Complete EN/PL visual learning environment and deterministic SEED Tutor.
- Portable graph projects and standalone Cargo export.
- Local Cargo Workspace Lab with public GitHub and crates.io integration.
- Explicit approvals, isolated execution, capability reporting, and trajectory records.
- Public documentation, contribution policy, security model, and CI release gate.

## v0.4 — Project confidence

- Project rename, duplicate, delete, backup, and explicit export/import.
- Versioned local project migrations with recovery before mutation.
- Dependency feature selection and `cargo tree` inspection.
- Better compiler-diagnostic navigation and terminal history.
- First-run environment diagnostics and copyable support report.

## v0.5 — Classroom and workshop readiness

- Teacher-authored lesson packs with a documented, versioned format.
- Resettable classroom workspaces and offline lesson bundles.
- Accessibility review and keyboard-only workflow coverage.
- Cross-distribution Linux verification and distributable release artifacts.

## Later, after security gates

- Optional GitHub authentication for private repositories.
- Optional crates.io authentication and publishing workflow.
- Additional host platforms and signed installers.
- Extension API for third-party learning nodes.

These items require explicit credential handling, threat-model updates, and dedicated tests. They
will not be added as shortcuts around the current safety model.
