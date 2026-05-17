import org.fragmentcolor.*

val torch = Light.spot(listOf(0.0f, 1.8f, 1.0f), listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
torch.setConeAngles(0.15, 0.4)

// Non-spot lights error.
val lamp = Light.point(listOf(0.0f, 0.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val unsupported = lamp.setConeAngles(0.15, 0.4)