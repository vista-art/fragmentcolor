package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Mesh, Vertex and
// Quad APIs, matching the call shapes of Mesh+Extensions.swift. Positions go
// straight in as numbers, so callers don't wrap every point in a Vertex.

// ── Vertex ────────────────────────────────────────────────────────────────────

/** The attribute key of a vertex normal, `"normal"`. */
val Vertex.Companion.NORMAL: String get() = "normal"

/** The attribute key of a vertex tangent, `"tangent"`. */
val Vertex.Companion.TANGENT: String get() = "tangent"

/** The attribute key of the first texture coordinate set, `"uv0"`. */
val Vertex.Companion.UV0: String get() = "uv0"

/** The attribute key of the second texture coordinate set, `"uv1"`. */
val Vertex.Companion.UV1: String get() = "uv1"

/** The attribute key of the first vertex color, `"color0"`. */
val Vertex.Companion.COLOR0: String get() = "color0"

/** The attribute key of the second vertex color, `"color1"`. */
val Vertex.Companion.COLOR1: String get() = "color1"

/** A vertex at [position], 1 to 4 numbers. */
operator fun Vertex.Companion.invoke(position: FloatArray): Vertex = Vertex(position.toList())

/** A vertex at [position], 1 to 4 numbers. */
@JvmName("invokeDoubles")
operator fun Vertex.Companion.invoke(position: List<Double>): Vertex = Vertex(readFloats(position, "Vertex: position"))

/** A vertex at [position], 1 to 4 numbers. */
operator fun Vertex.Companion.invoke(position: DoubleArray): Vertex = Vertex(readFloats(position, "Vertex: position"))

/** A vertex at [position] with the PBR shader's default attributes. */
fun Vertex.Companion.pbr(position: FloatArray): Vertex = pbr(position.toList())

/** A vertex at [position] with the PBR shader's default attributes. */
@JvmName("pbrDoubles")
fun Vertex.Companion.pbr(position: List<Double>): Vertex = pbr(readFloats(position, "Vertex.pbr: position"))

/** A vertex at [position] with the PBR shader's default attributes. */
fun Vertex.Companion.pbr(position: DoubleArray): Vertex = pbr(readFloats(position, "Vertex.pbr: position"))

// ── Mesh: vertices ────────────────────────────────────────────────────────────

/** Append a vertex at [position], 1 to 4 numbers. */
@JvmName("addVertexFloats")
fun Mesh.addVertex(position: List<Float>) {
    addVertex(Vertex(readPosition(position, "Mesh.addVertex: vertex")))
}

/** Append a vertex at [position], 1 to 4 numbers. */
@JvmName("addVertexDoubles")
fun Mesh.addVertex(position: List<Double>) {
    addVertex(Vertex(readPosition(position, "Mesh.addVertex: vertex")))
}

/** Append a vertex at [position], 1 to 4 numbers. */
fun Mesh.addVertex(position: FloatArray) {
    addVertex(position.toList())
}

/** Append a vertex at [position], 1 to 4 numbers. */
fun Mesh.addVertex(position: DoubleArray) {
    addVertex(position.toList())
}

/**
 * Append a vertex at [position] with a `"uv"` attribute,
 * as Rust's `(position, uv)` tuple does: 3 and 2 finite numbers.
 */
@JvmName("addVertexFloatsUv")
fun Mesh.addVertex(position: List<Float>, uv: List<Float>) {
    val point = readVector(position, 3, "Mesh.addVertex: vertex: position")
    val coordinates = readVector(uv, 2, "Mesh.addVertex: vertex: uv")
    addVertex(Vertex(point).set("uv", coordinates))
}

@JvmName("addVertexDoublesUv")
fun Mesh.addVertex(position: List<Double>, uv: List<Double>) {
    val point = readVector(position, 3, "Mesh.addVertex: vertex: position")
    val coordinates = readVector(uv, 2, "Mesh.addVertex: vertex: uv")
    addVertex(Vertex(point).set("uv", coordinates))
}

/**
 * Append a vertex at [position] with `"uv"` and `"color"` attributes,
 * as Rust's `(position, uv, color)` tuple does: 3, 2 and 4 finite numbers.
 */
@JvmName("addVertexFloatsUvColor")
fun Mesh.addVertex(position: List<Float>, uv: List<Float>, color: List<Float>) {
    val point = readVector(position, 3, "Mesh.addVertex: vertex: position")
    val coordinates = readVector(uv, 2, "Mesh.addVertex: vertex: uv")
    val tint = readVector(color, 4, "Mesh.addVertex: vertex: color")
    addVertex(Vertex(point).set("uv", coordinates).set("color", tint))
}

@JvmName("addVertexDoublesUvColor")
fun Mesh.addVertex(position: List<Double>, uv: List<Double>, color: List<Double>) {
    val point = readVector(position, 3, "Mesh.addVertex: vertex: position")
    val coordinates = readVector(uv, 2, "Mesh.addVertex: vertex: uv")
    val tint = readVector(color, 4, "Mesh.addVertex: vertex: color")
    addVertex(Vertex(point).set("uv", coordinates).set("color", tint))
}

/** Append a vertex for each position. */
@JvmName("addVerticesFloats")
fun Mesh.addVertices(positions: List<List<Float>>) {
    addVertices(positions.mapIndexed { index, position -> Vertex(readPosition(position, "Mesh.addVertices: vertex $index")) })
}

/** Append a vertex for each position. */
@JvmName("addVerticesDoubles")
fun Mesh.addVertices(positions: List<List<Double>>) {
    addVertices(positions.mapIndexed { index, position -> Vertex(readPosition(position, "Mesh.addVertices: vertex $index")) })
}

/** A mesh with a vertex at each position. */
@JvmName("fromVerticesFloats")
fun Mesh.Companion.fromVertices(positions: List<List<Float>>): Mesh =
    fromVertices(positions.mapIndexed { index, position -> Vertex(readPosition(position, "Mesh.fromVertices: vertex $index")) })

/** A mesh with a vertex at each position. */
@JvmName("fromVerticesDoubles")
fun Mesh.Companion.fromVertices(positions: List<List<Double>>): Mesh =
    fromVertices(positions.mapIndexed { index, position -> Vertex(readPosition(position, "Mesh.fromVertices: vertex $index")) })

// ── Mesh: instances ───────────────────────────────────────────────────────────

/** Append the per-instance attributes of a vertex, as Rust's `Instance::from(vertex)` does. */
fun Mesh.addInstance(vertex: Vertex) {
    addInstance(vertex.createInstance())
}

/**
 * Append an instance from attribute lists, in the map's order.
 * Throws [FragmentColorException] for a list that isn't 1 to 4 numbers.
 */
@JvmName("addInstanceDoubles")
fun Mesh.addInstance(attributes: Map<String, List<Double>>) {
    addInstance(attributes.entries.fold(Instance()) { instance, (key, value) -> instance.set(key, value) })
}

/** Append an instance from attribute lists, in the map's order. */
@JvmName("addInstanceFloats")
fun Mesh.addInstance(attributes: Map<String, List<Float>>) {
    addInstance(attributes.entries.fold(Instance()) { instance, (key, value) -> instance.set(key, value) })
}

/** Append an instance from attribute lists, in the map's order. */
@JvmName("addInstanceInts")
fun Mesh.addInstance(attributes: Map<String, List<Int>>) {
    addInstance(attributes.entries.fold(Instance()) { instance, (key, value) -> instance.set(key, value) })
}

/** Append the per-instance attributes of each vertex. */
@JvmName("addInstancesVertices")
fun Mesh.addInstances(vertices: List<Vertex>) {
    addInstances(vertices.map { it.createInstance() })
}

// ── Quad ──────────────────────────────────────────────────────────────────────

/** A quad from its clip-space corners [min] and [max], 2 numbers each. */
operator fun Quad.Companion.invoke(min: FloatArray, max: FloatArray): Quad = Quad(min.toList(), max.toList())

/** A quad from its clip-space corners [min] and [max], 2 numbers each. */
@JvmName("invokeDoubles")
operator fun Quad.Companion.invoke(min: List<Double>, max: List<Double>): Quad =
    Quad(readFloats(min, "Quad: min"), readFloats(max, "Quad: max"))
