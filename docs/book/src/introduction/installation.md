# Installation and Setup

Welcome to the `plrender` library setup guide. This section will guide you through the process of integrating `plrender` into your JavaScript or Python projects. By the end of this guide, you'll have the library set up and ready to use.

<div class="tab-container">
    <div class="tab active" data-target="js">JavaScript</div>
    <div class="tab" data-target="python">Python</div>
</div>

## 1. Installation

<div class="code-block active js">
You can include `plrender` in your project using npm:


```bash
npm install plrender
```

</div>

<div class="code-block python">
You can install `plrender` using pip:

```bash
pip install plrender
```

</div>

## 2. Usage

Once installed, you can import the library in your JavaScript or TypeScript files:

<div class="code-block active js">

```javascript
import load_wasm, { PLRender } from "plrender";
await load_wasm();

const plrender = new PLRender();

// add scene, target and entities

plrender.run();
```

</div>

<div class="code-block python">

```python
from plrender import PLRender
plrender = PLRender()

# add scene, target and entities

plrender.run()
```

</div>

## 3. Additional Resources

For more detailed examples and use-cases, refer to the [Usage Examples](./usage_examples.md) section.
