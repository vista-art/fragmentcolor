package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Scene API.
// The mobile binding takes a SceneObjectHandle enum (Model / Camera / Light)
// and dispatches internally; these overloads let callers pass the concrete
// types directly so example code reads scene.add(model) instead of
// scene.add(SceneObjectHandle.Model(model)).

/** Add a [Model] to the scene. */
fun Scene.add(model: Model) {
    add(SceneObjectHandle.Model(model))
}

/** Add a [Camera] to the scene. */
fun Scene.add(camera: Camera) {
    add(SceneObjectHandle.Camera(camera))
}

/** Add a [Light] to the scene. */
fun Scene.add(light: Light) {
    add(SceneObjectHandle.Light(light))
}

// add_to overloads: target a Pass by index or name, passing the concrete
// object type directly instead of wrapping it in PassTarget /
// SceneObjectHandle.

/** Add a [Model] to the Pass at [index]. */
fun Scene.addTo(index: Int, model: Model) {
    addTo(PassTarget.Index(index.toULong()), SceneObjectHandle.Model(model))
}

/** Add a [Camera] to the Pass at [index]. */
fun Scene.addTo(index: Int, camera: Camera) {
    addTo(PassTarget.Index(index.toULong()), SceneObjectHandle.Camera(camera))
}

/** Add a [Light] to the Pass at [index]. */
fun Scene.addTo(index: Int, light: Light) {
    addTo(PassTarget.Index(index.toULong()), SceneObjectHandle.Light(light))
}

/** Add a [Model] to the Pass named [name]. */
fun Scene.addTo(name: String, model: Model) {
    addTo(PassTarget.Name(name), SceneObjectHandle.Model(model))
}

/** Add a [Camera] to the Pass named [name]. */
fun Scene.addTo(name: String, camera: Camera) {
    addTo(PassTarget.Name(name), SceneObjectHandle.Camera(camera))
}

/** Add a [Light] to the Pass named [name]. */
fun Scene.addTo(name: String, light: Light) {
    addTo(PassTarget.Name(name), SceneObjectHandle.Light(light))
}
