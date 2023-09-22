import wgpu.backends.rs
from wgpu.utils import get_default_device
import numpy as np
from pathlib import Path
from PIL import Image

shader = Path('png.wgsl').read_text()

image = Image.open('composed.png')
array = np.asarray(image, dtype=np.uint8)

w, h, nz, nc = image.width, image.height, 4, 32

device = get_default_device()
cshader = device.create_shader_module(code=shader)

# Create textures and views
texture1 = device.create_texture(
    size=(w, h, 1),
    format=wgpu.TextureFormat.rgba8uint,
    dimension=wgpu.TextureDimension.d2,
    usage=wgpu.TextureUsage.TEXTURE_BINDING | wgpu.TextureUsage.COPY_DST,
)
texture2 = device.create_texture(
    size=(w, h, nc),
    format=wgpu.TextureFormat.rgba8uint,
    dimension=wgpu.TextureDimension.d3,
    usage=wgpu.TextureUsage.STORAGE_BINDING | wgpu.TextureUsage.COPY_SRC,
)
texture_view1 = texture1.create_view()
texture_view2 = texture2.create_view()

# Define bindings
# One can see here why we need 2 textures: one is readonly, one writeonly
bindings = [
    {"binding": 0, "resource": texture_view1},
    {"binding": 1, "resource": texture_view2},
]
binding_layouts = [
    {
        "binding": 0,
        "visibility": wgpu.ShaderStage.COMPUTE,
        "texture": {
            "sample_type": "uint",
            "view_dimension": wgpu.TextureDimension.d2,
        },
    },
    {
        "binding": 1,
        "visibility": wgpu.ShaderStage.COMPUTE,
        "storage_texture": {
            "access": wgpu.StorageTextureAccess.write_only,
            "format": wgpu.TextureFormat.rgba8uint,
            "view_dimension": wgpu.TextureDimension.d3,
        },
    },
]
bind_group_layout = device.create_bind_group_layout(entries=binding_layouts)
pipeline_layout = device.create_pipeline_layout(
    bind_group_layouts=[bind_group_layout]
)
bind_group = device.create_bind_group(
    layout=bind_group_layout, entries=bindings)

# Create a pipeline and run it
compute_pipeline = device.create_compute_pipeline(
    layout=pipeline_layout,
    compute={"module": cshader, "entry_point": "main"},
)
command_encoder = device.create_command_encoder()

device.queue.write_texture(
    {"texture": texture1},
    array,
    {"bytes_per_row": 4 * w, "rows_per_image": h},
    (w, h, 1),
)

compute_pass = command_encoder.begin_compute_pass()
compute_pass.set_pipeline(compute_pipeline)
# last 2 elements not used
compute_pass.set_bind_group(0, bind_group, [], 0, 999999)
compute_pass.dispatch_workgroups(w, h, 4)
compute_pass.end()
device.queue.submit([command_encoder.finish()])

data = device.queue.read_texture(
    {"texture": texture2},
    {"bytes_per_row": 4 * w, "rows_per_image": h},
    (w, h, nc),
)
data = np.frombuffer(data, dtype=np.uint8)
# inverted height and width produces correct orientation
data.shape = (nc, h, w, 4)

for i in range(32):
    output_image = Image.fromarray(data[i], mode="RGBA")
    output_image.save(f"out{i}.png")
