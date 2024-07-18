# TUSD Example 

Demonstrates how to use `ResumableUpload` to create a [TUSD protocol](https://tus.io/protocols/resumable-upload) client.

### Compile the example client

Build `wasm`

```
wasm-pack build -t nodejs
```

### Start `tusd` reference server

```
podman run \
    -d \
    --rm \
    --name tusd \
    -p 1080:1080 \
    docker.io/tusproject/tusd:v1.9
```

### Run the example client

```
node index.js
```
