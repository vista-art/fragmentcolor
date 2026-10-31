import { LogLevel, Renderer, Shader } from "fragmentcolor";

// Log naga's notes while this shader builds, then go back to the usual level.
Renderer.setLogLevel(LogLevel.Debug);
const shader = Shader.default();
Renderer.setLogLevel(LogLevel.Info);