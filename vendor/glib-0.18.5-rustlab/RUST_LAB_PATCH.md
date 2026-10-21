# Rust Lab security backport

This vendored crate is `glib` 0.18.5 from crates.io (registry checksum
`233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`) with
the minimal upstream fix for RUSTSEC-2024-0429 (gtk-rs-core pull request 1343):

- make the `VariantStrIter::impl_get` out pointer mutable;
- pass `&mut p` to `g_variant_get_child`.

Tauri's current Linux GTK3 dependency graph requires the 0.18 line. Remove this
override as soon as the Tauri stack supports `glib` 0.20 or newer.
