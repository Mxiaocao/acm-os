# ACM-OS architecture boundaries

This document records only the BUILD B0.1 compile-time ownership boundary.
Product behavior remains governed by the frozen SPEC, DESIGN, and PLAN.

```text
React / TypeScript
        |
        v
Thin Tauri IPC shell
        |
        v
Application -----------------------> Domain
        ^                              ^
        |                              |
Infrastructure -----------------------+
```

Rules enforced by the B0.1 scaffold:

- Domain is pure Rust and does not depend on Tauri, SQLx, filesystem, HTTP, or platform APIs.
- Application depends on Domain and owns business orchestration contracts.
- Infrastructure may implement Application ports and may depend on Domain, but it does not own workflows.
- Tauri is the composition root and thin typed IPC boundary.
- React does not receive database or generic filesystem authority.
- IPC DTOs live at the shell/frontend boundary and are not reused as Domain models.

The `foundation_status` command is intentionally non-product functionality. It exists only as a wiring probe for B0.1 and must not grow into a generic backend API.
