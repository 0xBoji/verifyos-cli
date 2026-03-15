# verifyOS Backend (Rust)

This service provides a clean, versioned HTTP API that accepts an `.ipa` or `.app` upload and returns a normalized scan report.

## Quick Start

From the repo root:

```bash
cargo run --manifest-path apps/backend/Cargo.toml
```

The API listens on `http://127.0.0.1:7070` by default.

Example request:

```bash
curl -X POST http://127.0.0.1:7070/api/v1/scan \
  -F "bundle=@/path/to/YourApp.ipa" \
  -F "profile=full"
```

## API

`POST /api/v1/scan`

- multipart `bundle` file field (required)
- `profile` form field: `basic` or `full` (optional)

Response: JSON report (same shape as `voc --format json`).

## Notes

- This module depends on the root `verifyOS` crate (`verifyos-cli`).
- It is initialized as a standalone crate for a future Cargo workspace split.
