#!/bin/bash
set -e

# Export OpenAPI specs and WS schemas from Rust tests (no server needed)
echo "Exporting OpenAPI specs and WS schemas..."
cargo test --workspace export_openapi_spec export_ws_schemas -- --nocapture 2>&1 | grep -E "^(running|test|Wrote)" || true

# Generate TypeScript clients from OpenAPI specs
for dir in modules/*/frontend; do
    if [ -f "$dir/orval.config.ts" ]; then
        echo "Generating API client for $dir..."
        cd "$dir" && bunx orval && cd - > /dev/null
    fi
done

# Generate TypeScript types from WS JSON Schemas
for dir in modules/*/frontend; do
    for schema in "$dir"/ws-*.schema.json; do
        [ -f "$schema" ] || continue
        base=$(basename "$schema" .schema.json)
        out="$dir/src/lib/${base}.gen.ts"
        echo "Generating WS types: $schema -> $out"
        bunx json-schema-to-typescript -i "$schema" -o "$out"
    done
done

echo "Done. Run 'cargo run -p forge_server' to build and start."
