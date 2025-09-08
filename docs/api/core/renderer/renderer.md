# Renderer

The [Renderer](https://fragmentcolor.org/api/renderer) is the main entry point for
[FragmentColor](https://fragmentcolor.org) and normally the first object you create.

It is used to render
[Shader](https://fragmentcolor.org/api/shader)s,
[Pass](https://fragmentcolor.org/api/pass)es, and
[Frame](https://fragmentcolor.org/api/frame)s
to a [Target](https://fragmentcolor.org/api/target) (canvas or window) or to a Bitmap.

The [Renderer](https://fragmentcolor.org/api/renderer) internals are lazily initialized
when the user creates a [Target](https://fragmentcolor.org/api/target) or renders a Bitmap.
This ensures the adapter and device are compatible with the target environment.

At the point of creation, we don't know if it will be used offscreen
or attached to a platform-specific Window or Canvas.

The API ensures the [Renderer](https://fragmentcolor.org/api/renderer) is usable when `render()` is called,
because
the `render()` method expects a Target as input. So, the user must call
`Renderer.create_target()` first, which initializes a window adapter, or
`Renderer.render_image()` which initializes an offscreen adapter.

## Example

```rust
use fragmentcolor::Renderer;
let renderer = Renderer::new();
```

## Methods

### async create_target(target: PLATFORM_SPECIFIC)

#### Rust

```rust
use fragmentcolor::Renderer;

# fn main() {
let renderer = Renderer::new();
# }
```

#### Python

```python
from fragmentcolor import Renderer

def main():
    renderer = Renderer.new()
# pub async fn create_target()
```

#### Javascript

```javascript
import { Renderer } from "fragmentcolor";

const canvas = document.createElement("canvas");

async function main() {
    const renderer = Renderer.new();
    await renderer.create_target(canvas);
}
```

#### Swift (TBD)

```swift
import FragmentColor

func main() {
    let renderer = Renderer.new()
}
```

##### pub async fn create_target()

#### Kotlin (TBD)

##### pub async fn create_target()

```kotlin
import fragmentcolor.Renderer

fun main() {
    val renderer = Renderer.new()
}
```
