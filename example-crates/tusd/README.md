# TUSD Example 

Demonstrates how to use `ResumableUpload` to create a [TUSD protocol](https://tus.io/protocols/resumable-upload) client.

## Quick Run

```sh
# podman-compose or docker-compose
podman-compose up
```

## Breakdown

### Compile the example client

Build `wasm`

```
wasm-pack build -t nodejs
```

### Transpile the js glue

Build

### Start `tusd` reference server

```
podman run \
    -d \
    --rm \
    --name tusd \
    -p 1080 \
    docker.io/tusproject/tusd:v1.9
```

### Run the example client

```
node dist/client.js
```
