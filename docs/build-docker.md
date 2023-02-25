# Build `recmd` using docker

## Standard Build

Prepare the Docker image with:

```bash
./scripts/recmd_docker_build_build.sh
```

Build `recmd` with:

```bash
./scripts/recmd_docker_build.sh
```

The build artifacts will be placed in `target_docker` directory.

You can also pass arbitrary arguments to the script. For example, for cleaning
the build directory:

```bash
./scripts/recmd_docker_build.sh cargo clean
```

## Static Build

Prepare the Docker image with:

```bash
./scripts/recmd_docker_build_static_build.sh
```

Build a static `recmd` with:

```bash
./scripts/recmd_docker_build_static.sh
```

The build artifacts will be placed in `target_docker_static` directory.

You can also pass arbitrary arguments to the script. For example, for cleaning
the build directory:

```bash
./scripts/recmd_docker_build_static.sh cargo clean
```

