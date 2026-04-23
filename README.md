# curl-gui

A desktop HTTP client built with Tauri + Svelte. Think curl, but with a GUI.

## Quick Start

```bash
# Install deps
pnpm install

# Dev mode (opens Tauri window)
pnpm tauri-dev

# Build release
pnpm tauri-build
```

## Testing

A built-in echo server is included for quick manual testing:

```bash
# Start the test server (port 8765)
pnpm test-server

# Or with a custom port:
python3 test_server.py 9999
```

Then in curl-gUI, hit `http://localhost:8765/echo` with any method + body.

## Project Structure

| Path | What |
|------|------|
| `src/` | Svelte frontend |
| `src-tauri/` | Rust backend |
| `src-tauri/src/main.rs` | HTTP request handler (libcurl) |
| `test_server.py` | Zero-dep Python echo server for testing |

## License

MIT
