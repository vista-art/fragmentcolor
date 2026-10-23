package org.fragmentcolor

import android.content.res.AssetManager
import androidx.test.platform.app.InstrumentationRegistry
import com.sun.jna.Library
import com.sun.jna.Native
import java.io.File
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Before
import org.junit.BeforeClass

// Base class of the healthcheck tests. The examples read fixtures such as
// `examples/assets/logo.png` relative to the repository root, as the Rust
// doctests do, so build.gradle packs those fixtures into the test APK and each
// test class runs from a copy of them. A test fails on any GPU error that wgpu
// reports, and a failing example names its `docs/api` page.
abstract class ExampleCase {
    /** Each example starts from the default registry, as each Rust doctest runs in a process of its own. */
    @Before
    fun useDefaultRegistry() {
        Shader.setRegistry("https://fragmentcolor.org/shaders/")
        takeGpuErrors()
    }

    /** Fails a test outside [example] that left a GPU error behind. */
    @After
    fun checkGpuErrors() {
        takeGpuErrors()?.let { throw AssertionError("GPU error: $it") }
    }

    /** Runs one example and fails the test with its `docs/api` page when it throws or raises a GPU error. */
    fun example(page: String, body: suspend () -> Unit) {
        runBlocking {
            try {
                body()
            } catch (error: Throwable) {
                throw AssertionError("$page: $error", error)
            }
        }
        takeGpuErrors()?.let { throw AssertionError("$page: GPU error: $it") }
    }

    companion object {
        /** The fixture directories that build.gradle packs into the test APK. */
        private val FIXTURES = listOf("examples", "path")

        /** The file where the library appends each GPU error that no error scope caught. */
        private val GPU_ERRORS by lazy {
            File(InstrumentationRegistry.getInstrumentation().context.cacheDir, "gpu-errors.log")
        }

        /** Copies the fixtures out of the test APK and moves the process into their root. */
        @BeforeClass
        @JvmStatic
        fun enterFixtures() {
            val context = InstrumentationRegistry.getInstrumentation().context
            val root = File(context.cacheDir, "fixtures")
            FIXTURES.forEach { copyAssets(context.assets, it, root) }
            check(LibC.INSTANCE.chdir(root.path) == 0) { "chdir to $root failed" }
            check(LibC.INSTANCE.setenv("FC_GPU_ERROR_LOG", GPU_ERRORS.path, 1) == 0) {
                "setenv FC_GPU_ERROR_LOG failed"
            }
        }

        /** Empties the GPU error log and returns what it held, or null when it held nothing. */
        private fun takeGpuErrors(): String? {
            val text = if (GPU_ERRORS.exists()) GPU_ERRORS.readText() else ""
            GPU_ERRORS.writeText("")
            return text.trim().ifEmpty { null }
        }

        private fun copyAssets(assets: AssetManager, path: String, root: File) {
            val children = assets.list(path).orEmpty()
            if (children.isNotEmpty()) {
                children.forEach { copyAssets(assets, "$path/$it", root) }
                return
            }
            val file = File(root, path)
            file.parentFile?.mkdirs()
            assets.open(path).use { input -> file.outputStream().use { input.copyTo(it) } }
        }
    }

    /** The C library, for the `chdir` and `setenv` that Java leaves out. */
    internal interface LibC : Library {
        fun chdir(path: String): Int

        fun setenv(name: String, value: String, overwrite: Int): Int

        companion object {
            val INSTANCE: LibC = Native.load("c", LibC::class.java)
        }
    }
}
