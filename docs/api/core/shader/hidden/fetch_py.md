# Shader.fetch (Python)

Python-specific example for `Shader.fetch`. The published wheels resolve
registry URLs and slugs offline against a shader library bundled into the
wheel, so this example runs without network access.

## Example

```python
from fragmentcolor import Shader

# Full registry URL.
shader = Shader.fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")

# Equivalent shorthand using the registry slug.
shader2 = Shader.fetch("sdf2d/circle")
```
