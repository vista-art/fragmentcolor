# Shader.fetch (Python)

Python-specific example for `Shader.fetch`. The canonical doc form fetches
a shader over HTTPS, which depends on (a) the wheel having TLS support
compiled in and (b) the shader URL being reachable from the CI runner.
The healthcheck wheel has TLS, but the registry endpoint is not always
live, so this stub exercises the API surface without leaning on the
network.

## Example

```python
import os, sys
# Skip the network round-trip in CI (no outbound network or no shader host).
# Set FC_ALLOW_NETWORK=1 to run the live URL fetch locally.
if os.environ.get('FC_ALLOW_NETWORK') != '1':
    raise SystemExit(0)

from fragmentcolor import Shader

# Single URL
shader = Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

# Registry slug
shader2 = Shader.fetch("sdf2d/circle")
```
