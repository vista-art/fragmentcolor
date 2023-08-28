// Basic syntax for macro definition:
macro_rules! example_struct_macro {
    ($t:ident) => {
        struct $t {
            pub x: f64,
            pub y: f64,
        }
    };
    // we can have as many expansion rules as we want
    () => {
        struct SomeDefaultStructName {
            pub x: f64,
            pub y: f64,
        }
    };
}

// example usage:
example_struct_macro!(A);
const MY_A: A = A { x: 1.0, y: 2.0 };
example_struct_macro!();
const MY_B: SomeDefaultStructName = SomeDefaultStructName { x: 1.0, y: 2.0 };

// MACRO CAPTURES:
//
// from: https://danielkeep.github.io/tlborm/book/mbe-macro-rules.html#captures
//
// Captures are written as a dollar ($) followed by an identifier,
// a colon (:), and finally the kind of capture, which must be
// one of the following:
//
// item: an item, like a function, struct, module, etc.
// block: a block (i.e. a block of statements and/or an expression, surrounded by braces)
// stmt: a statement
// pat: a pattern
// expr: an expression
// ty: a type
// ident: an identifier
// path: a path (e.g. foo, ::std::mem::replace, transmute::<_, int>, …)
// meta: a meta item; the things that go inside #[...] and #![...] attributes
// tt: a single token tree

// For example, here is a macro which captures its input as an expression:
macro_rules! one_expression {
    ($e:expr) => {
        println!("Result of expression is {}", $e);
    };
}
macro_rules! multiply_add {
    ($a:expr, $b:expr, $c:expr) => {
        $a * ($b + $c)
    };
}

// Example usage:
fn print_expressions() {
    one_expression!(1 + 2); // prints "Result of expression is 3"
    let v = multiply_add!(1, 2, 3); // returns 7
}

// REPETITIONS:
//
//from: https://danielkeep.github.io/tlborm/book/mbe-macro-rules.html#repetitions
//
// Patterns can contain repetitions. These allow a sequence of tokens to be matched.
// These have the general form $ ( ... ) sep rep.
//
//    $ is a literal dollar token.
//    ( ... ) is the paren-grouped pattern being repeated.
//    sep is an optional separator token. Common examples are ,, and ;.
//    rep is the required repeat control. Currently, this can be either:
//          * (indicating zero or more repeats)
//        or
//          + (indicating one or more repeats).
//        You cannot write "zero or one" or any other more specific counts or ranges.

// For example, below is a macro which formats each element as a string.
// It matches zero or more comma-separated expressions and expands
// to an expression that constructs a vector.
macro_rules! vec_strs {
    (
        // Start a repetition:
        $(
            // Each repeat must contain an expression...
            $element:expr
        )
        // ...separated by commas...
        ,
        // ...zero or more times.
        *
    ) => {
        // Enclose the expansion in a block so that we can use
        // multiple statements.
        {
            let mut v = Vec::new();

            // Start a repetition:
            $(
                // Each repeat will contain the following statement, with
                // $element replaced with the corresponding expression.
                v.push(format!("{}", $element));
            )*

            v
        }
    };
}

// Example usage:
fn print_vec_strs() {
    let v = vec_strs![1, 2, 3];
    assert_eq!(v, ["1", "2", "3"]);
}

// ----------

// See also:
// https://jstrong.dev/posts/2020/productive-rust-implementing-traits-with-macros

// @TODO implement Circle manually first, then
//       copy the working code to a macro definition
//       and use it to generate any Renderable struct
//
// uncomment this when you decide to start implementing:
//
// macro_rules! renderable {
//     (
//         ($t:ident, $u:ident)
//     ) => {
//         impl Renderable for $t {
//             ... // @TODO
//         }
// }
