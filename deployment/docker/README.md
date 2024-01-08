## Install using Docker

### Prerequisites

- Docker installed on the system

### Install

1. Create a directory for Wave Autoscale:

```bash showLineNumbers
mkdir wave-autoscale
cd wave-autoscale
```

2. Download the required configuration and definition files:

```bash showLineNumbers
wget https://stclab-inc.github.io/wave-autoscale/config/wave-config.yaml
wget https://stclab-inc.github.io/wave-autoscale/config/wave-definition.yaml
```

3. Run Wave Autoscale using Docker:

```bash showLineNumbers
docker run -d \
    -v $(pwd):/app/config \
    -p 3024:3024 \
    -p 3025:3025 \
    --name wave-autoscale \
    waveautoscale/wave-autoscale:latest

# (3024 is the port for the Wave Autoscale API and 3025 is the port for the Wave Autoscale UI)
```

### Verify

Check if the Wave Autoscale container is running:

```bash showLineNumbers
docker ps | grep wave-autoscale
```

You should see the Wave Autoscale container running.

### Edit Configuration

1. Edit the `wave-config.yaml`:

```bash showLineNumbers
vi wave-config.yaml
```

2. After making changes, restart the Wave Autoscale container for the changes to take effect:

```bash showLineNumbers
docker restart wave-autoscale
```

### Edit Definitions

1. Edit the `wave-definition.yaml`:

```bash showLineNumbers
vi wave-definition.yaml
```

Note: Add or modify definitions according to your needs.

2. After making changes to the `wave-definition.yaml`, restart the Wave Autoscale container for the changes to take effect:

```bash showLineNumbers
docker restart wave-autoscale
```

### Uninstall

To stop and remove the Wave Autoscale container:

```bash showLineNumbers
docker stop wave-autoscale
docker rm wave-autoscale
```