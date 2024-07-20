
# TUS Web

TUS protocol client for web and node.

## Test

Start the reference server:

```
podman run \
    -d \
    --rm \
    --name tusd \
    -p 1080:1080 \
    docker.io/tusproject/tusd:v1.9
```

Run tests

```
wasm-pack test --node
```
