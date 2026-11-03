from fragmentcolor import LogLevel, Renderer, Shader

# Log naga's notes while this shader builds, then go back to the usual level.
Renderer.set_log_level(LogLevel.Debug)
shader = Shader.default()
Renderer.set_log_level(LogLevel.Info)