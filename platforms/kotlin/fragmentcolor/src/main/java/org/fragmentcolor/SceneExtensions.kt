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
