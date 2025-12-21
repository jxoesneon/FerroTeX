# License Selection (Required)

FerroTeX does not currently ship with a license. Before publishing or distributing code, you should choose one.

This file exists to make the choice explicit and documented.

## Recommended Default (common for Rust tooling)

- **Apache-2.0 OR MIT (dual license)**

**Why**

- Maximizes adoption (friendly to industry and academia)
- Compatible with many ecosystems
- Common in Rust and developer tooling

## Alternatives

### MPL-2.0

- Weak copyleft (file-level)
- Good if you want modifications to core files to remain open, while still allowing broader use

### GPL-3.0

- Strong copyleft
- Best if you want derivatives to remain open-source, but reduces adoption in some environments

### AGPL-3.0

- Strong copyleft including network use
- Appropriate if you deploy a hosted service and want modifications shared

## Decision Record

Choose one option and record it here.

- Selected license: **MIT OR Apache-2.0**
- Rationale: **Maximizes adoption and compatibility with the Rust ecosystem (common for Rust tooling and language servers).**
- Date: **2025-12-19**

After you decide, create a top-level `LICENSE` file and (if dual-licensing) include `LICENSE-MIT` and `LICENSE-APACHE`.
