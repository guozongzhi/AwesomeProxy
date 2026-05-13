# LiteLLM Sidecar Binary

Place the prebuilt `litellm-sidecar` executable in this directory before running a packaged proxy build:

```bash
cp /path/to/litellm-sidecar src-tauri/binaries/litellm-sidecar
chmod +x src-tauri/binaries/litellm-sidecar
```

The Rust core starts it with:

```bash
litellm-sidecar --config ~/.awesomeproxy/config.yaml --port 4000
```
