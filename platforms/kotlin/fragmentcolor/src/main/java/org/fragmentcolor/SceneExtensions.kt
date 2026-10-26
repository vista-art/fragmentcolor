package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Scene API.
// The mobile binding takes a SceneObjectHandle enum (Model / Camera / Light)
// and dispatches internally; these overloads let callers pass the concrete
// types directly so example code reads scene.add(model) instead of
// scene.add(SceneObjectHandle.Model(model)). Every mutator returns the scene,
// so calls chain.

/**
 * Load a scene from the bytes of a `.glb` file, as JS and Python do.
 * [cameras] and [lights] decide whether the file's own cameras and lights load.
 */
fun Scene.Companion.load(bytes: ByteArray, cameras: Boolean = true, lights: Boolean = true): Scene =
    loadBytes(bytes, cameras, lights)

/** Add a [Model] to the scene. */
fun Scene.add(model: Model): Scene = add(SceneObjectHandle.Model(model))

/** Add a [Camera] to the scene. */
fun Scene.add(camera: Camera): Scene = add(SceneObjectHandle.Camera(camera))

/** Add a [Light] to the scene. */
fun Scene.add(light: Light): Scene = add(SceneObjectHandle.Light(light))

/** Set the scene ambient color from `[r, g, b]`. */
fun Scene.ambient(color: FloatArray): Scene = ambient(color.toList())

/** Set the scene ambient color from `[r, g, b]`. */
@JvmName("ambientDoubles")
fun Scene.ambient(color: List<Double>): Scene = ambient(readFloats(color, "Scene.ambient"))

/** Set the scene ambient color from `[r, g, b]`. */
fun Scene.ambient(color: DoubleArray): Scene = ambient(readFloats(color, "Scene.ambient"))

// add_to overloads: target a Pass by index or name, passing the concrete
// object type directly instead of wrapping it in PassTarget /
// SceneObjectHandle.

/** Add a [Model] to the Pass at [index]. */
fun Scene.addTo(index: UInt, model: Model): Scene = addTo(PassTarget.Index(index), SceneObjectHandle.Model(model))

/** Add a [Camera] to the Pass at [index]. */
fun Scene.addTo(index: UInt, camera: Camera): Scene = addTo(PassTarget.Index(index), SceneObjectHandle.Camera(camera))

/** Add a [Light] to the Pass at [index]. */
fun Scene.addTo(index: UInt, light: Light): Scene = addTo(PassTarget.Index(index), SceneObjectHandle.Light(light))

/** Add a [Model] to the Pass named [name]. */
fun Scene.addTo(name: String, model: Model): Scene = addTo(PassTarget.Name(name), SceneObjectHandle.Model(model))

/** Add a [Camera] to the Pass named [name]. */
fun Scene.addTo(name: String, camera: Camera): Scene = addTo(PassTarget.Name(name), SceneObjectHandle.Camera(camera))

/** Add a [Light] to the Pass named [name]. */
fun Scene.addTo(name: String, light: Light): Scene = addTo(PassTarget.Name(name), SceneObjectHandle.Light(light))

/** Replace the whole pass graph from an array of passes. */
fun Scene.setPasses(passes: Array<Pass>) {
    setPasses(passes.toList())
}
