package org.fragmentcolor

import android.content.res.AssetManager
import androidx.test.platform.app.InstrumentationRegistry
import com.sun.jna.Library
import com.sun.jna.Native
import java.io.File
import kotlinx.coroutines.runBlocking
import org.junit.BeforeClass

// Base class of the generated example tests. The examples read fixtures such
// as `examples/assets/logo.png` relative to the repository root, as the Rust
// doctests do, so build.gradle packs those fixtures into the test APK and each
// test class runs from a copy of them. A failing example names its `docs/api` page.
open class ExampleCase {
    /** Runs one example and fails the test with its `docs/api` page when it throws. */
    fun example(page: String, body: suspend () -> Unit) {
        runBlocking {
            try {
                body()
            } catch (error: Throwable) {
                throw AssertionError("$page: $error", error)
            }
        }
    }

    companion object {
        /** The fixture directories that build.gradle packs into the test APK. */
        private val FIXTURES = listOf("examples", "path")

        /** Copies the fixtures out of the test APK and moves the process into their root. */
        @BeforeClass
        @JvmStatic
        fun enterFixtures() {
            val context = InstrumentationRegistry.getInstrumentation().context
            val root = File(context.cacheDir, "fixtures")
            FIXTURES.forEach { copyAssets(context.assets, it, root) }
            check(LibC.INSTANCE.chdir(root.path) == 0) { "chdir to $root failed" }
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

    /** The C library, for the `chdir` that Java leaves out. */
    internal interface LibC : Library {
        fun chdir(path: String): Int

        companion object {
            val INSTANCE: LibC = Native.load("c", LibC::class.java)
        }
    }
}
