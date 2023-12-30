# PLRender Macros

This crate provides **Procedural Macros** for PLRender and its platform-specific bindings.

Right now, it provides the `wrap_py()` and `wrap_wasm()` macros for the `plrender-py` and `plrender-wasm` crates.

After every successful `plrender` build, an API map is saved in the `../generated` directory. Then, the build system triggers a build in this crate to automatically update the macros.

## What are Procedural Macros

> This description is an adapted excerpt from this excellent [article about procedural nacros](https://developerlife.com/2022/03/30/rust-proc-macro) by [Nazmul Idris](https://developerlife.com/about-me). Check it out if you want to learn more.

Rust has two types of macros to generate code at compile time: **Declarative** and **Procedural**.

**Declarative macros** can be declared inline in the program and are easier to use. They are great for simple use cases, but have some limitations (eg: they can't work with generics). You can learn more about them in the [Little Book of Rust Macros](https://veykril.github.io/tlborm/introduction.html) and in the [Rust Book](https://doc.rust-lang.org/book/ch19-06-macros.html).

**Procedural macros** must be declared in an external crate and imported by your program. They are a powerful way to extend the Rust compiler and provide plugins that you can use to extend the language.

Here are the key benefits of procedural macros:

- Minimize the amount of manual work you have to do in order to generate boilerplate code 🎉.
- You can create your own domain specific language like React JSX in Rust 🎉.

|         Macro type         |                                              Capabilities & limitations                                              |
| :------------------------: | :------------------------------------------------------------------------------------------------------------------: |
|        Declarative         |          Can't handle generics, patterns capture items as wholes and can't be broken down in the macro body          |
| Procedural - function like |              Operates on the code passed inside parenthesis of invocation to produce new token stream.               |
|    Procedural - derive     | Can't touch token stream of annotated struct or enum, only add new token stream below; can declare helper attributes |
|   Procedural - attribute   |                Like function-like, replaces token stream of annotated item (not just struct or enum)                 |

## Types of Procedural Macros

```rust
#[proc_macro]
pub fn my_fn_like_proc_macro(input: TokenStream) -> TokenStream {
    // 1. Use syn to parse the input tokens into a syntax tree.
    // 2. Use quote to generate new tokens based on what we parsed.
    // 3. Return the generated tokens.
    input
}

#[proc_macro_derive(MyDerive)]
pub fn my_derive_proc_macro(input: TokenStream) -> TokenStream {
    // 1. Use syn to parse the input tokens into a syntax tree.
    // 2. Generate new tokens based on the syntax tree. This is additive to the `enum` or
    //    `struct` that is annotated (it doesn't replace them).
    // 3. Return the generated tokens.
}

#[proc_macro_attribute]
pub fn log_entry_and_exit(args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. Use syn to parse the args & input tokens into a syntax tree.
    // 2. Generate new tokens based on the syntax tree. This will replace whatever `item` is
    //    annotated w/ this attribute proc macro.
    // 3. Return the generated tokens.
    input
}
```
