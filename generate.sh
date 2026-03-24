#!/bin/bash
set -e

# Export OpenAPI specs from Rust tests (no server needed)
echo "Exporting OpenAPI specs..."
cargo test --workspace export_openapi_spec -- --nocapture 2>&1 | grep -E "^(running|test|Wrote)" || true

# Generate TypeScript clients
for dir in modules/*/frontend; do
    if [ -f "$dir/orval.config.ts" ]; then
        echo "Generating API client for $dir..."
        cd "$dir" && bunx orval && cd - > /dev/null
    fi
done

echo "Done. Run 'cargo run -p forge_server' to build and start."
