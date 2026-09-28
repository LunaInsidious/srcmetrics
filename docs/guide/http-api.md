# HTTP API and web UI

```sh
srcmetrics serve [--bind 127.0.0.1] [--port 8080]
```

The server listens on `127.0.0.1` by default and has **no authentication**; it is meant for local
use. `--port 0` picks a free port; the address is printed on stderr.

| Endpoint | |
|---|---|
| `GET /` | Web UI: paste code, give it a file name and see its metrics |
| `POST /api/analyze` | Analyze one file |

## `POST /api/analyze`

```sh
curl -X POST http://127.0.0.1:8080/api/analyze \
  -H 'Content-Type: application/json' \
  -d '{"filename": "a.py", "source": "def f(x):\n    return x\n"}'
```

The extension of `filename` selects the language. The response is an analysis result in the
[output format](./output) with one file; `repository` and `commit` are `null`.

Unsupported extensions and syntax errors return status 400 with `{"error": "…"}`.
