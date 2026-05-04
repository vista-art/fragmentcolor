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