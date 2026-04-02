#!/bin/bash
set -e

# Export OpenAPI spec from Rust tests
echo "Exporting OpenAPI spec..."
cargo test export_openapi_spec -- --nocapture 2>&1 | grep -E "^(running|test|Wrote)" || true

# Generate TypeScript client
echo "Generating API client..."
cd frontend && bunx orval && cd -

echo "Done. Run 'cargo run' to build and start."
