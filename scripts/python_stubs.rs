mod python_stubs {
    //! Python type stub generator.
    //!
    //! Writes `platforms/python/fragmentcolor/fragmentcolor.pyi` from two sources:
    //!
    //! - **Signatures** come from the pyo3 bindings in the shared source model
    //!   ([`super::reflect`]), seen as the Python build compiles them: every
    //!   `#[pyclass]`, `#[pymethods]` and `#[pyfunction]` (also through
    //!   `#[cfg_attr(python, ...)]`), with each Rust type mapped to a Python hint.
    //!   `#[doc(hidden)]` methods stay in: Python can still call them, and some,
    //!   like the `ScreenRegion` constructors, are how users build arguments.
    //!   The classes and functions registered in `src/platforms/python.rs` are
    //!   the module surface. A class that a method only returns is not
    //!   importable, so it appears under a private `_Name`.
    //! - **Docstrings** come from the `#[lsp_doc(...)]` page of each item. A
    //!   `hidden/<stem>_py.md` override defers to the canonical `<stem>.md` for
    //!   its prose. The example is the Python file the website export writes
    //!   under `platforms/python/examples/`, the same code the healthcheck runs.
    //!
    //! `Py<PyAny>` parameters carry no type information, so `HINT_OVERRIDES`
    //! names the accepted types for them. An override that no longer matches a
    //! binding fails the build, which keeps the table honest.

    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;

    use quote::ToTokens;
    use syn::{Attribute, FnArg, GenericArgument, Meta, Pat, PathArguments, ReturnType, Type};

    use super::reflect::{self, Build, Reflection, TypeKind};

    const STUB_PATH: &str = "platforms/python/fragmentcolor/fragmentcolor.pyi";
    const MODULE_INIT: &str = "src/platforms/python.rs";
    const EXAMPLES_DIR: &str = "platforms/python/examples";

    /// Hints for parameters (or `return`) whose Rust type says nothing useful.
    /// Keyed by Python class (empty for module functions), Python method, and
    /// parameter name.
    const HINT_OVERRIDES: &[(&str, &str, &str, &str)] = &[
        ("Renderer", "render", "renderable", "_Renderable"),
        (
            "Renderer",
            "render",
            "target",
            "RenderCanvasTarget | TextureTarget",
        ),
        ("Renderer", "load", "renderable", "_Renderable"),
        ("Renderer", "create_target", "rendercanvas", "Any"),
        ("Renderer", "create_texture", "input", "_TextureSource"),
        ("Renderer", "read_texture", "texture_id", "TextureId | int"),
        (
            "Renderer",
            "unregister_texture",
            "texture_id",
            "TextureId | int",
        ),
        ("Pass", "add", "object", "Model | Camera | Light"),
        ("Pass", "require", "deps", "_Renderable"),
        ("Pass", "set_target", "target", "TextureTarget | Texture"),
        (
            "Pass",
            "set_depth_target",
            "target",
            "TextureTarget | Texture",
        ),
        ("Scene", "load", "source", "str | bytes"),
        ("Scene", "add", "object", "Model | Camera | Light"),
        ("Scene", "add_to", "target", "int | str"),
        ("Scene", "add_to", "object", "Model | Camera | Light"),
        ("Shader", "new", "input", "str | Sequence[str]"),
        ("Shader", "fetch", "input", "str | Sequence[str]"),
        ("Shader", "set", "value", "_UniformValue"),
        ("Shader", "get", "return", "_UniformValue"),
        ("Vertex", "new", "position", "Sequence[float]"),
        ("Vertex", "pbr", "position", "Sequence[float]"),
        (
            "Vertex",
            "set",
            "value",
            "float | int | Sequence[float] | Sequence[int]",
        ),
        (
            "Instance",
            "set",
            "value",
            "float | int | Sequence[float] | Sequence[int]",
        ),
        (
            "Mesh",
            "from_vertices",
            "vertices",
            "Sequence[Vertex | Sequence[float]]",
        ),
        ("Mesh", "add_vertex", "vertex", "Vertex | Sequence[float]"),
        (
            "Mesh",
            "add_vertices",
            "vertices",
            "Sequence[Vertex | Sequence[float]]",
        ),
        ("Mesh", "add_instance", "item", "Instance | Vertex"),
        (
            "Mesh",
            "add_instances",
            "items",
            "Sequence[Instance | Vertex]",
        ),
        ("Mesh", "set_indices", "indices", "Sequence[int]"),
        (
            "Texture",
            "set_sampler_options",
            "options",
            "SamplerOptions | Mapping[str, Any]",
        ),
        (
            "Texture",
            "write",
            "data",
            "bytes | Sequence[int] | _NDArrayU8",
        ),
        (
            "Texture",
            "write_region",
            "data",
            "bytes | Sequence[int] | _NDArrayU8",
        ),
        (
            "Texture",
            "write_region",
            "region",
            "Sequence[int] | Mapping[str, int] | None",
        ),
        (
            "Mipmap",
            "build",
            "bytes",
            "bytes | Sequence[int] | _NDArrayU8",
        ),
    ];

    /// Rust types that derive `FromPyObject` / `IntoPyObject` as an untagged
    /// union, with the hint each one takes as a parameter and as a return value.
    const DERIVED_HINTS: &[(&str, &str, &str)] = &[(
        "VertexValue",
        "float | int | Sequence[float] | Sequence[int]",
        "float | int | list[float] | list[int]",
    )];

    /// Aliases written at the top of the stub, used by mapped types and overrides.
    const ALIASES: &[(&str, &str)] = &[
        ("_NDArrayU8", "npt.NDArray[numpy.uint8]"),
        ("_PySize", "Sequence[float] | Mapping[str, float]"),
        (
            "_Renderable",
            "Shader | Pass | Scene | Sequence[Shader | Pass | Scene]",
        ),
        (
            "_TextureSource",
            "str | bytes | bytearray | Sequence[int] | _NDArrayU8 | Mipmap",
        ),
        (
            "_UniformValue",
            "float | int | bool | Sequence[float] | Sequence[int] | bytes | bytearray | memoryview | Texture",
        ),
    ];

    const PY_KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ];

    #[derive(Default)]
    struct Class {
        py_name: String,
        subclass: bool,
        lsp_doc: Option<String>,
        fields: Vec<Field>,
        variants: Vec<String>,
    }

    struct Field {
        name: String,
        ty: Type,
        settable: bool,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Init,
        Instance,
        Static,
        Class,
        Getter,
        Setter,
        ClassAttr,
        Function,
    }

    struct Method {
        py_name: String,
        kind: Kind,
        params: Vec<(String, Type)>,
        signature: Option<String>,
        ret: Option<Type>,
        lsp_doc: Option<String>,
    }

    #[derive(Default)]
    struct Bindings {
        /// Rust ident → pyclass.
        classes: BTreeMap<String, Class>,
        /// Rust ident of the impl's self type → methods, in source order.
        methods: BTreeMap<String, Vec<Method>>,
        /// Rust fn ident → `#[pyfunction]`.
        functions: BTreeMap<String, Method>,
    }

    struct Module {
        classes: Vec<String>,
        functions: Vec<String>,
        exceptions: Vec<String>,
    }

    /// Context shared by the renderers: maps Rust class idents to stub names.
    struct Ctx<'a> {
        stub_names: &'a BTreeMap<String, String>,
        unknown: std::cell::RefCell<BTreeSet<String>>,
        referenced: std::cell::RefCell<BTreeSet<String>>,
    }

    pub fn generate(root: &Path, model: &Reflection) {
        println!("cargo::rerun-if-changed={}", EXAMPLES_DIR);

        let bindings = scan(model);
        let module = read_module(root);

        // Registered classes keep their Python name. A class that is only
        // returned from a method can't be imported, so its stub name is private.
        let mut stub_names: BTreeMap<String, String> = BTreeMap::new();
        for rust in &module.classes {
            let Some(class) = bindings.classes.get(rust) else {
                panic!(
                    "python_stubs: `{rust}` is registered in {MODULE_INIT} but has no #[pyclass]"
                );
            };
            stub_names.insert(rust.clone(), class.py_name.clone());
        }
        for (rust, class) in &bindings.classes {
            stub_names
                .entry(rust.clone())
                .or_insert_with(|| format!("_{}", class.py_name));
        }
        let ctx = Ctx {
            stub_names: &stub_names,
            unknown: Default::default(),
            referenced: Default::default(),
        };

        let mut overrides_used: BTreeSet<(String, String, String)> = BTreeSet::new();

        let mut functions = String::new();
        for rust in &module.functions {
            let Some(method) = bindings.functions.get(rust) else {
                panic!(
                    "python_stubs: `{rust}` is registered in {MODULE_INIT} but has no #[pyfunction]"
                );
            };
            functions.push_str(&render_method(
                root,
                ("", ""),
                method,
                &ctx,
                &mut overrides_used,
                0,
            ));
        }

        // Registered classes first, then any class their signatures return.
        let mut rendered: BTreeMap<String, String> = BTreeMap::new();
        let mut pending: Vec<String> = module.classes.iter().rev().cloned().collect();
        pending.extend(std::mem::take(&mut *ctx.referenced.borrow_mut()));
        while let Some(rust) = pending.pop() {
            if rendered.contains_key(&rust) {
                continue;
            }
            let Some(class) = bindings.classes.get(&rust) else {
                continue;
            };
            let body = render_class(root, &bindings, &rust, class, &ctx, &mut overrides_used);
            rendered.insert(rust.clone(), body);
            let referenced = std::mem::take(&mut *ctx.referenced.borrow_mut());
            pending.extend(referenced.into_iter().filter(|r| !rendered.contains_key(r)));
        }
        let mut order: Vec<&String> = module.classes.iter().collect();
        let mut extra: Vec<&String> = rendered
            .keys()
            .filter(|r| !module.classes.contains(r))
            .collect();
        extra.sort_by_key(|rust| &stub_names[*rust]);
        order.extend(extra);

        let stale: Vec<String> = HINT_OVERRIDES
            .iter()
            .filter(|(c, m, p, _)| {
                !overrides_used.contains(&(c.to_string(), m.to_string(), p.to_string()))
            })
            .map(|(c, m, p, _)| format!("{c}.{m}({p})"))
            .collect();
        if !stale.is_empty() {
            panic!(
                "python_stubs: HINT_OVERRIDES entries match no Python binding: {}",
                stale.join(", ")
            );
        }
        let unknown = ctx.unknown.borrow();
        if !unknown.is_empty() {
            let list: Vec<&str> = unknown.iter().map(String::as_str).collect();
            println!(
                "cargo::warning=python_stubs: no Python hint for Rust types {}; the stub uses Any",
                list.join(", ")
            );
        }

        let mut out = String::new();
        out.push_str(
            "# Generated by build.rs (scripts/python_stubs.rs) from the pyo3 bindings in src/\n",
        );
        out.push_str("# and the docs in docs/api. Do not edit by hand.\n");
        out.push_str("from typing import Any, ClassVar, Mapping, Sequence, final\n\n");
        out.push_str("import numpy\n");
        out.push_str("import numpy.typing as npt\n");
        out.push_str("from typing_extensions import TypeAlias\n\n");
        out.push_str("__all__ = [\n");
        let public = module
            .exceptions
            .iter()
            .chain(module.classes.iter().map(|rust| &stub_names[rust]))
            .chain(module.functions.iter());
        for name in public {
            out.push_str(&format!("    \"{name}\",\n"));
        }
        out.push_str("]\n\n");
        for (name, value) in ALIASES {
            out.push_str(&format!("{name}: TypeAlias = {value}\n"));
        }
        out.push('\n');
        for exception in &module.exceptions {
            out.push_str(&format!("class {exception}(Exception): ...\n\n"));
        }
        for rust in order {
            out.push_str(&rendered[rust]);
            out.push('\n');
        }
        out.push_str(&functions);
        let out = format!("{}\n", out.trim_end());

        let path = root.join(STUB_PATH);
        if let Err(e) = super::meta::write_if_changed(&path, &out) {
            panic!("python_stubs: failed to write {}: {e}", path.display());
        }
    }

    // ---------------------------------------------------------------------
    // Scanning
    // ---------------------------------------------------------------------

    fn scan(model: &Reflection) -> Bindings {
        let build = Build::Python;
        let mut bindings = Bindings::default();
        for ty in model.types_in(build) {
            let Some(args) = pyclass_args(&ty.attrs) else {
                continue;
            };
            let mut class = Class {
                py_name: arg_value(&args, "name").unwrap_or_else(|| ty.ident.clone()),
                subclass: split_top_level(&args).iter().any(|a| a == "subclass"),
                lsp_doc: reflect::lsp_doc(&ty.attrs),
                ..Default::default()
            };
            match ty.kind {
                TypeKind::Struct => {
                    for field in &ty.fields {
                        let Some(args) = pyo3_args(&field.attrs) else {
                            continue;
                        };
                        let flags = split_top_level(&args);
                        if !flags.iter().any(|f| f == "get") {
                            continue;
                        }
                        let Some(ident) = &field.ident else {
                            continue;
                        };
                        class.fields.push(Field {
                            name: arg_value(&args, "name").unwrap_or_else(|| ident.clone()),
                            ty: field.ty.clone(),
                            settable: flags.iter().any(|f| f == "set"),
                        });
                    }
                }
                TypeKind::Enum => {
                    for variant in &ty.variants {
                        if !variant.is_unit || !variant.cfg.holds(build) {
                            continue;
                        }
                        let name = pyo3_args(&variant.attrs)
                            .and_then(|a| arg_value(&a, "name"))
                            .unwrap_or_else(|| variant.ident.clone());
                        class.variants.push(name);
                    }
                }
            }
            bindings.classes.insert(ty.ident.clone(), class);
        }

        // Methods keep source order within a file, and files keep path order.
        let mut impls: Vec<_> = model.binding_impls(build, "pymethods").collect();
        impls.sort_by(|a, b| a.file.cmp(&b.file));
        for imp in impls {
            for f in imp.fns.iter().filter(|f| f.cfg.holds(build)) {
                bindings
                    .methods
                    .entry(imp.self_ident.clone())
                    .or_default()
                    .push(read_method(&f.attrs, &f.sig, false));
            }
        }
        for f in model.binding_fns(build, "pyfunction") {
            bindings
                .functions
                .insert(f.sig.ident.to_string(), read_method(&f.attrs, &f.sig, true));
        }
        bindings
    }

    fn read_method(attrs: &[Attribute], sig: &syn::Signature, function: bool) -> Method {
        let rust_name = sig.ident.to_string();
        let pyo3 = pyo3_args(attrs).unwrap_or_default();
        let mut kind = if function {
            Kind::Function
        } else {
            Kind::Instance
        };
        let mut explicit_name = arg_value(&pyo3, "name");
        for meta in reflect::metas(attrs, Build::Python) {
            let ident = reflect::meta_ident(&meta);
            match ident.as_str() {
                "new" => kind = Kind::Init,
                "staticmethod" => kind = Kind::Static,
                "classmethod" => kind = Kind::Class,
                "classattr" => kind = Kind::ClassAttr,
                "getter" | "setter" => {
                    kind = if ident == "getter" {
                        Kind::Getter
                    } else {
                        Kind::Setter
                    };
                    if let Meta::List(list) = &meta {
                        explicit_name = Some(list.tokens.to_string().trim().to_string());
                    }
                }
                _ => {}
            }
        }
        let py_name = match (kind, explicit_name) {
            (Kind::Init, _) => "__new__".to_string(),
            (_, Some(name)) => name,
            (Kind::Getter, None) => rust_name
                .strip_prefix("get_")
                .unwrap_or(&rust_name)
                .to_string(),
            (Kind::Setter, None) => rust_name
                .strip_prefix("set_")
                .unwrap_or(&rust_name)
                .to_string(),
            (_, None) => rust_name.clone(),
        };

        let mut params = Vec::new();
        let mut skipped_cls = false;
        for input in &sig.inputs {
            let FnArg::Typed(typed) = input else {
                continue;
            };
            if kind == Kind::Class && !skipped_cls {
                skipped_cls = true;
                continue;
            }
            if is_python_token(&typed.ty) || is_self_receiver(&typed.ty) {
                continue;
            }
            let name = match typed.pat.as_ref() {
                Pat::Ident(p) => p.ident.to_string(),
                _ => format!("arg{}", params.len()),
            };
            params.push((name, (*typed.ty).clone()));
        }
        let ret = match &sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some((**ty).clone()),
        };
        Method {
            py_name,
            kind,
            params,
            signature: arg_value(&pyo3, "signature"),
            ret,
            lsp_doc: reflect::lsp_doc(attrs),
        }
    }

    /// Reads `add_class::<T>`, `wrap_pyfunction!(f, ...)` and `m.add("Error", ...)`
    /// from the `#[pymodule]` initializer, in registration order.
    fn read_module(root: &Path) -> Module {
        let path = root.join(MODULE_INIT);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("python_stubs: cannot read {}: {e}", path.display()));
        let code: String = text
            .lines()
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");

        let mut module = Module {
            classes: Vec::new(),
            functions: Vec::new(),
            exceptions: Vec::new(),
        };
        let mut rest = code.as_str();
        while let Some(at) = rest.find("add_class::<") {
            rest = &rest[at + "add_class::<".len()..];
            let end = rest.find('>').unwrap_or(rest.len());
            let path = rest[..end].trim();
            module
                .classes
                .push(path.rsplit("::").next().unwrap_or(path).trim().to_string());
        }
        let mut rest = code.as_str();
        while let Some(at) = rest.find("wrap_pyfunction!(") {
            rest = &rest[at + "wrap_pyfunction!(".len()..];
            let end = rest.find(',').unwrap_or(rest.len());
            module.functions.push(rest[..end].trim().to_string());
        }
        let mut rest = code.as_str();
        while let Some(at) = rest.find(".add(") {
            rest = &rest[at + ".add(".len()..];
            if let Some(stripped) = rest.trim_start().strip_prefix('"')
                && let Some(end) = stripped.find('"')
            {
                module.exceptions.push(stripped[..end].to_string());
            }
        }
        module
    }

    // ---------------------------------------------------------------------
    // Attributes
    // ---------------------------------------------------------------------

    fn pyclass_args(attrs: &[Attribute]) -> Option<String> {
        reflect::metas(attrs, Build::Python)
            .iter()
            .find(|m| reflect::meta_ident(m) == "pyclass")
            .map(reflect::meta_args)
    }

    /// Merged arguments of every `#[pyo3(...)]` on the item.
    fn pyo3_args(attrs: &[Attribute]) -> Option<String> {
        let parts: Vec<String> = reflect::metas(attrs, Build::Python)
            .iter()
            .filter(|m| reflect::meta_ident(m) == "pyo3")
            .map(reflect::meta_args)
            .collect();
        (!parts.is_empty()).then(|| parts.join(" , "))
    }

    /// Splits `a , b = "x" , c = (1 , 2)` on commas outside brackets and strings.
    fn split_top_level(args: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut depth = 0i32;
        let mut in_str = false;
        let mut current = String::new();
        let mut prev = '\0';
        for ch in args.chars() {
            match ch {
                '"' if prev != '\\' => in_str = !in_str,
                '(' | '[' | '{' if !in_str => depth += 1,
                ')' | ']' | '}' if !in_str => depth -= 1,
                ',' if !in_str && depth == 0 => {
                    parts.push(current.trim().to_string());
                    current.clear();
                    prev = ch;
                    continue;
                }
                _ => {}
            }
            current.push(ch);
            prev = ch;
        }
        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }
        parts
    }

    /// Value of `key = value` in an attribute argument list. String values
    /// lose their quotes, parenthesized values their outer parentheses.
    fn arg_value(args: &str, key: &str) -> Option<String> {
        for part in split_top_level(args) {
            let Some((k, v)) = part.split_once('=') else {
                continue;
            };
            if k.trim() != key {
                continue;
            }
            let v = v.trim();
            if let Some(s) = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                return Some(s.to_string());
            }
            if let Some(s) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                return Some(s.trim().to_string());
            }
            return Some(v.to_string());
        }
        None
    }

    // ---------------------------------------------------------------------
    // Types
    // ---------------------------------------------------------------------

    fn last_ident(path: &syn::Path) -> String {
        path.segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default()
    }

    fn type_args(ty: &Type) -> Vec<&Type> {
        let Type::Path(p) = ty else {
            return Vec::new();
        };
        let Some(seg) = p.path.segments.last() else {
            return Vec::new();
        };
        let PathArguments::AngleBracketed(args) = &seg.arguments else {
            return Vec::new();
        };
        args.args
            .iter()
            .filter_map(|a| match a {
                GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .collect()
    }

    fn strip_refs(ty: &Type) -> &Type {
        match ty {
            Type::Reference(r) => strip_refs(&r.elem),
            Type::Paren(p) => strip_refs(&p.elem),
            Type::Group(g) => strip_refs(&g.elem),
            _ => ty,
        }
    }

    fn type_ident(ty: &Type) -> String {
        match strip_refs(ty) {
            Type::Path(p) => last_ident(&p.path),
            _ => String::new(),
        }
    }

    fn is_python_token(ty: &Type) -> bool {
        type_ident(ty) == "Python"
    }

    /// `slf: PyRef<'_, Self>` and friends stand in for `self`.
    fn is_self_receiver(ty: &Type) -> bool {
        matches!(
            type_ident(ty).as_str(),
            "PyRef" | "PyRefMut" | "Bound" | "Py"
        ) && type_args(strip_refs(ty))
            .first()
            .is_some_and(|t| type_ident(t) == "Self")
    }

    fn is_u8(ty: &Type) -> bool {
        type_ident(ty) == "u8"
    }

    /// Python hint for a Rust type. `param` selects the accepting form
    /// (`Sequence`) over the returned form (`list`).
    fn py_type(ty: &Type, param: bool, self_name: &str, ctx: &Ctx) -> String {
        let ty = strip_refs(ty);
        match ty {
            Type::Tuple(t) if t.elems.is_empty() => "None".into(),
            Type::Tuple(t) => {
                let items: Vec<String> = t
                    .elems
                    .iter()
                    .map(|e| py_type(e, param, self_name, ctx))
                    .collect();
                format!("tuple[{}]", items.join(", "))
            }
            Type::Array(a) => sequence_of(&a.elem, param, self_name, ctx),
            Type::Slice(s) => sequence_of(&s.elem, param, self_name, ctx),
            Type::Path(p) => {
                let ident = last_ident(&p.path);
                let args = type_args(ty);
                let first = || args.first().copied();
                match ident.as_str() {
                    "Self" => self_name.to_string(),
                    "bool" => "bool".into(),
                    "f32" | "f64" => "float".into(),
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                    | "i64" | "i128" | "isize" => "int".into(),
                    "String" | "str" | "char" | "Cow" => "str".into(),
                    "PathBuf" | "Path" => "str".into(),
                    "Option" => match first() {
                        Some(inner) => format!("{} | None", py_type(inner, param, self_name, ctx)),
                        None => "Any".into(),
                    },
                    "Vec" => match first() {
                        Some(inner) => sequence_of(inner, param, self_name, ctx),
                        None => "Any".into(),
                    },
                    "HashMap" | "BTreeMap" => {
                        let k = args
                            .first()
                            .map(|t| py_type(t, param, self_name, ctx))
                            .unwrap_or("Any".into());
                        let v = args
                            .get(1)
                            .map(|t| py_type(t, param, self_name, ctx))
                            .unwrap_or("Any".into());
                        if param {
                            format!("Mapping[{k}, {v}]")
                        } else {
                            format!("dict[{k}, {v}]")
                        }
                    }
                    "Result" | "PyResult" | "Py" | "Bound" | "Borrowed" | "PyRef" | "PyRefMut"
                    | "Box" | "Arc" | "Rc" => match first() {
                        Some(inner) => py_type(inner, param, self_name, ctx),
                        None => "Any".into(),
                    },
                    "PyAny" | "PyObject" => "Any".into(),
                    "PyDict" => "dict[str, Any]".into(),
                    "PyList" => "list[Any]".into(),
                    "PyTuple" => "tuple[Any, ...]".into(),
                    "PyBytes" => "bytes".into(),
                    "PyString" => "str".into(),
                    "PyType" => "type".into(),
                    "PyFloat" => "float".into(),
                    "PyInt" | "PyLong" => "int".into(),
                    "PyBool" => "bool".into(),
                    "PyArray1" | "PyArray2" | "PyArray3" | "PyArrayDyn" | "PyReadonlyArray1"
                    | "PyReadonlyArray2" | "PyReadonlyArray3" | "PyReadonlyArrayDyn" => {
                        match first().map(type_ident).as_deref() {
                            Some("u8") => "_NDArrayU8".into(),
                            Some("f32") => "npt.NDArray[numpy.float32]".into(),
                            Some("f64") => "npt.NDArray[numpy.float64]".into(),
                            Some("u32") => "npt.NDArray[numpy.uint32]".into(),
                            Some("i32") => "npt.NDArray[numpy.int32]".into(),
                            _ => "npt.NDArray[Any]".into(),
                        }
                    }
                    _ if ALIASES
                        .iter()
                        .any(|(name, _)| name.strip_prefix('_') == Some(&ident)) =>
                    {
                        format!("_{ident}")
                    }
                    _ if DERIVED_HINTS.iter().any(|(name, _, _)| *name == ident) => DERIVED_HINTS
                        .iter()
                        .find(|(name, _, _)| *name == ident)
                        .map(|(_, p, r)| if param { p.to_string() } else { r.to_string() })
                        .unwrap_or_default(),
                    _ => match ctx.stub_names.get(&ident) {
                        Some(stub_name) => {
                            ctx.referenced.borrow_mut().insert(ident.clone());
                            stub_name.clone()
                        }
                        None => {
                            ctx.unknown.borrow_mut().insert(ident);
                            "Any".into()
                        }
                    },
                }
            }
            _ => {
                ctx.unknown
                    .borrow_mut()
                    .insert(ty.to_token_stream().to_string());
                "Any".into()
            }
        }
    }

    /// pyo3 returns `Vec<u8>` and `[u8; N]` as `bytes`, and accepts them from
    /// `bytes` or any sequence of ints. Other element types round-trip as lists.
    fn sequence_of(elem: &Type, param: bool, self_name: &str, ctx: &Ctx) -> String {
        if is_u8(elem) {
            return if param {
                "bytes | Sequence[int]".into()
            } else {
                "bytes".into()
            };
        }
        let inner = py_type(elem, param, self_name, ctx);
        if param {
            format!("Sequence[{inner}]")
        } else {
            format!("list[{inner}]")
        }
    }

    // ---------------------------------------------------------------------
    // Rendering
    // ---------------------------------------------------------------------

    fn render_class(
        root: &Path,
        bindings: &Bindings,
        rust: &str,
        class: &Class,
        ctx: &Ctx,
        used: &mut BTreeSet<(String, String, String)>,
    ) -> String {
        let stub_name = ctx.stub_names[rust].as_str();
        let names = (class.py_name.as_str(), stub_name);
        // pyo3 classes reject subclassing unless they opt in with `subclass`.
        let mut out = if class.subclass {
            String::new()
        } else {
            "@final\n".to_string()
        };
        out.push_str(&format!("class {stub_name}:\n"));
        let mut body = String::new();
        if let Some(doc) = class_doc(root, class) {
            body.push_str(&docstring(&doc, 4));
        }
        for variant in &class.variants {
            body.push_str(&format!("    {variant}: ClassVar[{stub_name}]\n"));
        }
        for field in &class.fields {
            let hint = py_type(&field.ty, false, stub_name, ctx);
            if field.settable {
                body.push_str(&format!("    {}: {hint}\n", field.name));
            } else {
                body.push_str(&format!(
                    "    @property\n    def {}(self) -> {hint}: ...\n",
                    field.name
                ));
            }
        }
        let mut getters: BTreeSet<&str> = BTreeSet::new();
        for method in bindings.methods.get(rust).into_iter().flatten() {
            if method.kind == Kind::Getter {
                getters.insert(&method.py_name);
            }
            if method.kind == Kind::Setter && !getters.contains(method.py_name.as_str()) {
                continue;
            }
            body.push_str(&render_method(root, names, method, ctx, used, 4));
        }
        if body.is_empty() {
            out.truncate(out.len() - 1);
            out.push_str(" ...\n");
        } else {
            out.push_str(&body);
        }
        out
    }

    /// `names` is the owner's Python name (for `HINT_OVERRIDES`) and its stub
    /// name (for `Self`). Module functions pass two empty strings.
    fn render_method(
        root: &Path,
        names: (&str, &str),
        method: &Method,
        ctx: &Ctx,
        used: &mut BTreeSet<(String, String, String)>,
        indent: usize,
    ) -> String {
        let pad = " ".repeat(indent);
        let (class, self_name) = names;
        // Overrides name the method as the docs do: the constructor is `new`.
        let lookup_name = if method.kind == Kind::Init {
            "new"
        } else {
            method.py_name.as_str()
        };
        let mut hint_for = |param: &str| -> Option<String> {
            HINT_OVERRIDES
                .iter()
                .find(|(c, m, p, _)| *c == class && *m == lookup_name && *p == param)
                .map(|(c, m, p, hint)| {
                    used.insert((c.to_string(), m.to_string(), p.to_string()));
                    hint.to_string()
                })
        };

        let mut params: Vec<String> = Vec::new();
        match method.kind {
            Kind::Instance | Kind::Getter | Kind::Setter => params.push("self".into()),
            Kind::Init | Kind::Class => params.push("cls".into()),
            Kind::Static | Kind::ClassAttr | Kind::Function => {}
        }
        let typed: Vec<(String, String)> = method
            .params
            .iter()
            .map(|(name, ty)| {
                let hint = hint_for(name).unwrap_or_else(|| py_type(ty, true, self_name, ctx));
                (py_param_name(name), hint)
            })
            .collect();
        match &method.signature {
            Some(signature) => {
                for part in split_top_level(signature) {
                    let part = part.trim();
                    if part == "*" || part == "/" {
                        params.push(part.to_string());
                    } else if let Some(name) = part.strip_prefix("**") {
                        params.push(format!("**{}: Any", py_param_name(name.trim())));
                    } else if let Some(name) = part.strip_prefix('*') {
                        params.push(format!("*{}: Any", py_param_name(name.trim())));
                    } else {
                        let (name, default) = match part.split_once('=') {
                            Some((n, _)) => (n.trim(), " = ..."),
                            None => (part, ""),
                        };
                        let name = py_param_name(name);
                        let hint = typed
                            .iter()
                            .find(|(n, _)| *n == name)
                            .map(|(_, h)| h.clone())
                            .unwrap_or_else(|| "Any".into());
                        params.push(format!("{name}: {hint}{default}"));
                    }
                }
            }
            None => {
                for (name, hint) in &typed {
                    params.push(format!("{name}: {hint}"));
                }
            }
        }

        let ret = if method.kind == Kind::Init {
            self_name.to_string()
        } else if method.kind == Kind::Setter {
            "None".to_string()
        } else {
            hint_for("return").unwrap_or_else(|| match &method.ret {
                Some(ty) => py_type(ty, false, self_name, ctx),
                None => "None".into(),
            })
        };

        let mut out = String::new();
        if method.kind == Kind::ClassAttr {
            out.push_str(&format!("{pad}{}: ClassVar[{ret}]\n", method.py_name));
            return out;
        }
        match method.kind {
            Kind::Static => out.push_str(&format!("{pad}@staticmethod\n")),
            Kind::Class => out.push_str(&format!("{pad}@classmethod\n")),
            Kind::Getter => out.push_str(&format!("{pad}@property\n")),
            Kind::Setter => out.push_str(&format!("{pad}@{}.setter\n", method.py_name)),
            _ => {}
        }
        let head = format!(
            "{pad}def {}({}) -> {ret}:",
            method.py_name,
            params.join(", ")
        );
        match method.lsp_doc.as_deref().and_then(|p| method_doc(root, p)) {
            Some(doc) => {
                out.push_str(&head);
                out.push('\n');
                out.push_str(&docstring(&doc, indent + 4));
            }
            None => {
                out.push_str(&head);
                out.push_str(" ...\n");
            }
        }
        out
    }

    fn py_param_name(name: &str) -> String {
        let name = name.trim();
        if PY_KEYWORDS.contains(&name) {
            format!("{name}_")
        } else {
            name.to_string()
        }
    }

    fn docstring(text: &str, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let escaped = text.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
        let mut out = format!("{pad}\"\"\"\n");
        for line in escaped.lines() {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str(&pad);
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str(&format!("{pad}\"\"\"\n"));
        out
    }

    // ---------------------------------------------------------------------
    // Docs
    // ---------------------------------------------------------------------

    fn class_doc(root: &Path, class: &Class) -> Option<String> {
        if let Some(path) = &class.lsp_doc {
            return method_doc(root, path);
        }
        let docs_root = root.join("docs/api");
        let dir_name = super::docs::object_dir_name(&class.py_name);
        let dir = super::docs::find_object_dir(&docs_root, &dir_name)?;
        let page = dir.join(format!("{dir_name}.md"));
        let rel = page
            .strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .replace('\\', "/");
        method_doc(root, &rel)
    }

    /// Prose of the page an `lsp_doc` path names, followed by the Python
    /// example the website export generated for it.
    fn method_doc(root: &Path, lsp_path: &str) -> Option<String> {
        let page = canonical_page(root, lsp_path);
        let md = fs::read_to_string(root.join(&page)).ok()?;
        let mut prose = page_prose(&md);
        if let Some(example) =
            example_path(&page).and_then(|p| fs::read_to_string(root.join(p)).ok())
        {
            let example = example.trim_start_matches(['\n', '\r']).trim_end();
            if !example.trim().is_empty() {
                prose.push_str("\n\nExample:\n\n```python\n");
                prose.push_str(example);
                prose.push_str("\n```");
            }
        }
        (!prose.trim().is_empty()).then_some(prose)
    }

    /// `<obj>/hidden/<stem>_py.md` → `<obj>/<stem>.md` when that page exists.
    fn canonical_page(root: &Path, lsp_path: &str) -> String {
        let path = Path::new(lsp_path);
        let parent = path.parent();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if parent.and_then(|p| p.file_name()).and_then(|n| n.to_str()) == Some("hidden")
            && let (Some(obj_dir), Some(base)) =
                (parent.and_then(Path::parent), stem.strip_suffix("_py"))
        {
            let candidate = obj_dir.join(format!("{base}.md"));
            if root.join(&candidate).exists() {
                return candidate.to_string_lossy().replace('\\', "/");
            }
        }
        lsp_path.to_string()
    }

    /// Generated example for a canonical page. Object pages use the
    /// CamelCase object name as the file stem, method pages their own stem.
    fn example_path(page: &str) -> Option<String> {
        let rel = page.strip_prefix("docs/api/")?;
        let path = Path::new(rel);
        let stem = path.file_stem()?.to_str()?;
        let dir = path.parent()?;
        let dir_name = dir.file_name()?.to_str()?;
        if dir_name == "hidden" {
            return None;
        }
        let file = if stem == dir_name {
            super::docs::dir_to_object_name(stem)
        } else {
            stem.to_string()
        };
        let dir = dir.to_string_lossy().replace('\\', "/");
        Some(format!("{EXAMPLES_DIR}/{dir}/{file}.py"))
    }

    /// Everything above `## Example`, without the H1 title line.
    fn page_prose(md: &str) -> String {
        let mut lines = Vec::new();
        let mut seen_title = false;
        for line in md.lines() {
            let t = line.trim_start();
            if t.starts_with("## Example") {
                break;
            }
            if !seen_title && t.starts_with("# ") {
                seen_title = true;
                continue;
            }
            lines.push(line);
        }
        lines.join("\n").trim().to_string()
    }
}
