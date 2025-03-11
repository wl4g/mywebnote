# MyWebNote

## Quick Start

- Building

```bash
docker build --platform=amd64 -t wl4g/mywebnote:latest .
```

- Running with Docker

```bash
docker run -d -p 18888:18888 -e MYWEBNOTE__DB__TYPE=SQLite wl4g/mywebnote:latest
```
