mod python_stubs {
    //! Python type stub generator.
    //!
    //! Writes `platforms/python/fragmentcolor/fragmentcolor.pyi` from two sources:
    //!
    //! - **Signatures** come from the pyo3 bindings in the shared source model
    //!   ([`super::reflect`]), seen as the Python build compiles them: every
    //!   `#[pyclass]`, `#[pymethods]` and `#[pyfunction]` (also through
    //!   `#[cfg_attr(python, ...)]`), with each Rust type mapped to a Python hint.
    //!   A documented class (one with `#[lsp_doc]`) leaves out its
    //!   `#[doc(hidden)]` methods, which are binding plumbing. A value type
    //!   with no doc page, such as `Size` or `ScreenRegion`, keeps them,
    //!   since its constructors are how users build arguments.
    //!   The classes and functions registered in `src/platforms/python.rs` are
    //!   the module surface. A class that a method only returns is not
    //!   importable, so it appears under a private `_Name`.
    //! - **Docstrings** come from the `#[lsp_doc(...)]` page of each item. A
    //!   `hidden/<stem>_py.md` override defers to the canonical `<stem>.md` for
    //!   its prose. The example is the Python file the website export writes
    //!   under `platforms/python/examples/`, the same code the healthcheck runs.
    //!
    //! A binding's own Rust type gives each hint. Parameters that pyo3 takes as
    //! an untyped `Py<PyAny>` inherit the type of the canonical method the
    //! binding documents (same `lsp_doc` page), expanded through the crate's
    //! conversions: `Into<X>` reads every `From` impl for `X`, `TryInto<X>`
    //! every `TryFrom`, a trait bound every implementor, and a
    //! `derive(FromPyObject)` enum its variants. Only types Python can see make
    //! it into a hint. A union of three or more types that two parameters share
    //! becomes a private alias named after its Rust source.

    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;

    use quote::ToTokens;
    use syn::{Attribute, FnArg, GenericArgument, Meta, Pat, PathArguments, ReturnType, Type};

    use super::reflect::{
        self, Build, PythonModule, Reflection, TypeKind, arg_value, split_top_level,
    };

    const STUB_PATH: &str = "platforms/python/fragmentcolor/fragmentcolor.pyi";
    const EXAMPLES_DIR: &str = "platforms/python/examples";

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
        /// Rust ident of the impl's self type; empty for module functions.
        owner: String,
        rust_name: String,
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

    /// Context shared by the renderers.
    struct Ctx<'a> {
        root: &'a Path,
        model: &'a Reflection,
        /// Rust ident of each pyclass → its name in the stub.
        stub_names: &'a BTreeMap<String, String>,
        /// Python class name → Rust ident, so a core type finds the pyclass
        /// that stands in for it (`TextureTarget` → `PyTextureTarget`).
        faces: &'a BTreeMap<String, String>,
        unknown: RefCell<BTreeSet<String>>,
        /// Documented untyped values that found no canonical counterpart.
        unmatched: RefCell<BTreeSet<String>>,
        referenced: RefCell<BTreeSet<String>>,
        /// Traits and types currently being expanded, to stop recursion.
        expanding: RefCell<Vec<String>>,
        /// Alias candidates: Rust source name → rendered union.
        aliases: RefCell<BTreeMap<String, String>>,
        /// Classes Python code can hold: registered ones and any a binding
        /// returns. `None` while that set is being worked out.
        obtainable: RefCell<Option<BTreeSet<String>>>,
    }

    pub fn generate(root: &Path, model: &Reflection) {
        println!("cargo::rerun-if-changed={}", EXAMPLES_DIR);

        let bindings = scan(model);
        let module = PythonModule::read(root);

        // Registered classes keep their Python name. A class that is only
        // returned from a method can't be imported, so its stub name is private.
        let mut stub_names: BTreeMap<String, String> = BTreeMap::new();
        for rust in &module.classes {
            let Some(class) = bindings.classes.get(rust) else {
                panic!(
                    "python_stubs: `{rust}` is registered in {} but has no #[pyclass]",
                    PythonModule::FILE
                );
            };
            stub_names.insert(rust.clone(), class.py_name.clone());
        }
        for (rust, class) in &bindings.classes {
            stub_names
                .entry(rust.clone())
                .or_insert_with(|| format!("_{}", class.py_name));
        }
        let faces: BTreeMap<String, String> = bindings
            .classes
            .iter()
            .map(|(rust, class)| (class.py_name.clone(), rust.clone()))
            .collect();
        let ctx = Ctx {
            root,
            model,
            stub_names: &stub_names,
            faces: &faces,
            unknown: Default::default(),
            unmatched: Default::default(),
            referenced: Default::default(),
            expanding: Default::default(),
            aliases: Default::default(),
            obtainable: Default::default(),
        };

        // Python holds only classes it can construct or receive; an internal
        // pyclass never appears in a parameter hint.
        for method in bindings
            .methods
            .values()
            .flatten()
            .chain(bindings.functions.values())
        {
            value_hint(method, None, Mode::Return, "", &ctx);
        }
        for class in bindings.classes.values() {
            for field in &class.fields {
                expand(&field.ty, Mode::Return, &Scope::default(), "", &ctx);
            }
        }
        let mut obtainable = std::mem::take(&mut *ctx.referenced.borrow_mut());
        obtainable.extend(module.classes.iter().cloned());
        *ctx.obtainable.borrow_mut() = Some(obtainable);
        ctx.unknown.borrow_mut().clear();
        ctx.unmatched.borrow_mut().clear();

        let mut functions = String::new();
        for rust in &module.functions {
            let Some(method) = bindings.functions.get(rust) else {
                panic!(
                    "python_stubs: `{rust}` is registered in {} but has no #[pyfunction]",
                    PythonModule::FILE
                );
            };
            functions.push_str(&render_method(root, "", method, &ctx, 0));
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
            let body = render_class(root, &bindings, &rust, class, &ctx);
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

        let unmatched = ctx.unmatched.borrow();
        if !unmatched.is_empty() {
            let list: Vec<&str> = unmatched.iter().map(String::as_str).collect();
            println!(
                "cargo::warning=python_stubs: no canonical value for {}; the stub uses Any",
                list.join(", ")
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
        out.push_str(
            "from typing import Any, ClassVar, Mapping, Sequence, SupportsFloat, SupportsIndex, final\n\n",
        );
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
        let mut body = String::new();
        // FragmentColorError is the root, and every other exception the module adds subclasses it.
        for exception in &module.exceptions {
            let base = match exception.as_str() {
                "FragmentColorError" => "Exception",
                _ => "FragmentColorError",
            };
            body.push_str(&format!("class {exception}({base}): ...\n\n"));
        }
        for rust in order {
            body.push_str(&rendered[rust]);
            body.push('\n');
        }
        body.push_str(&functions);

        // A shared union becomes an alias; one used once stays inline.
        for (name, union) in ctx.aliases.borrow().iter() {
            let token = placeholder(name);
            let alias = format!("_{name}");
            let shared = body.matches(&token).count() >= 2;
            if shared && !stub_names.values().any(|v| *v == alias) {
                out.push_str(&format!("{alias}: TypeAlias = {union}\n"));
                body = body.replace(&token, &alias);
            } else {
                body = body.replace(&token, union);
            }
        }
        out.push('\n');
        out.push_str(&body);
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
            let class_flags = split_top_level(&args);
            let get_all = class_flags.iter().any(|a| a == "get_all");
            let set_all = class_flags.iter().any(|a| a == "set_all");
            let mut class = Class {
                py_name: reflect::exported_type_name(&ty.attrs, &ty.ident, Build::Python)
                    .unwrap_or_else(|| ty.ident.clone()),
                subclass: class_flags.iter().any(|a| a == "subclass"),
                lsp_doc: reflect::lsp_doc(&ty.attrs),
                ..Default::default()
            };
            match ty.kind {
                TypeKind::Struct => {
                    for field in &ty.fields {
                        let args = pyo3_args(&field.attrs).unwrap_or_default();
                        let flags = split_top_level(&args);
                        if !get_all && !flags.iter().any(|f| f == "get") {
                            continue;
                        }
                        let Some(ident) = &field.ident else {
                            continue;
                        };
                        class.fields.push(Field {
                            name: arg_value(&args, "name").unwrap_or_else(|| ident.clone()),
                            ty: field.ty.clone(),
                            settable: set_all || flags.iter().any(|f| f == "set"),
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
            let documented = bindings
                .classes
                .get(&imp.self_ident)
                .is_some_and(|class| class.lsp_doc.is_some());
            let shown = |f: &&reflect::FnDef| !(documented && hides_docs(&f.attrs));
            for f in imp.fns.iter().filter(|f| f.cfg.holds(build)).filter(shown) {
                bindings
                    .methods
                    .entry(imp.self_ident.clone())
                    .or_default()
                    .push(read_method(&imp.self_ident, &f.attrs, &f.sig, false));
            }
        }
        for f in model.binding_fns(build, "pyfunction") {
            bindings.functions.insert(
                f.sig.ident.to_string(),
                read_method("", &f.attrs, &f.sig, true),
            );
        }
        bindings
    }

    /// Whether `#[doc(hidden)]` marks the item as internal.
    fn hides_docs(attrs: &[Attribute]) -> bool {
        reflect::metas(attrs, Build::Python)
            .iter()
            .any(|m| reflect::meta_ident(m) == "doc" && reflect::meta_args(m).contains("hidden"))
    }

    fn read_method(
        owner: &str,
        attrs: &[Attribute],
        sig: &syn::Signature,
        function: bool,
    ) -> Method {
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
            owner: owner.to_string(),
            rust_name,
            py_name,
            kind,
            params,
            signature: arg_value(&pyo3, "signature"),
            ret,
            lsp_doc: reflect::lsp_doc(attrs),
        }
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

    /// Untyped pyo3 values such as `Py<PyAny>` or `&Bound<'_, PyAny>`, also
    /// inside `PyResult` or `Option`.
    fn is_untyped(ty: &Type) -> bool {
        let ty = strip_refs(ty);
        let Type::Path(p) = ty else {
            return false;
        };
        match last_ident(&p.path).as_str() {
            "PyAny" | "PyObject" => true,
            "Py" | "Bound" | "Borrowed" | "Result" | "PyResult" | "Option" => {
                type_args(ty).first().is_some_and(|inner| is_untyped(inner))
            }
            _ => false,
        }
    }

    // ---------------------------------------------------------------------
    // Hints
    // ---------------------------------------------------------------------

    /// Which side of a call a hint describes. Parameters accept any sequence
    /// or mapping; return values are concrete lists and dicts.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Param,
        Return,
    }

    /// One member of a Python union.
    #[derive(Clone, PartialEq, Eq)]
    enum Atom {
        Name(String),
        Seq(Vec<Atom>),
        Tuple(Vec<Vec<Atom>>),
        Map(Vec<Atom>, Vec<Atom>),
        Null,
    }

    /// Generic parameters in scope and the trait bounds each one carries.
    #[derive(Default)]
    struct Scope {
        bounds: BTreeMap<String, Vec<syn::TypeParamBound>>,
    }

    fn scope_of(generics: &[&syn::Generics]) -> Scope {
        let mut scope = Scope::default();
        for g in generics {
            for param in &g.params {
                if let syn::GenericParam::Type(tp) = param {
                    scope
                        .bounds
                        .entry(tp.ident.to_string())
                        .or_default()
                        .extend(tp.bounds.iter().cloned());
                }
            }
            for pred in g.where_clause.iter().flat_map(|w| w.predicates.iter()) {
                let syn::WherePredicate::Type(pt) = pred else {
                    continue;
                };
                let Type::Path(bounded) = &pt.bounded_ty else {
                    continue;
                };
                if let Some(ident) = bounded.path.get_ident() {
                    scope
                        .bounds
                        .entry(ident.to_string())
                        .or_default()
                        .extend(pt.bounds.iter().cloned());
                }
            }
        }
        scope
    }

    /// Bounds that say nothing about which types fit.
    const MARKER_TRAITS: &[&str] = &[
        "Sized",
        "Clone",
        "Copy",
        "Send",
        "Sync",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "Hash",
        "Unpin",
        "Display",
        "ToString",
    ];

    fn rank(atom: &Atom) -> u8 {
        match atom {
            Atom::Name(n) => match n.as_str() {
                "bool" => 0,
                "int" => 1,
                "float" => 2,
                "str" => 3,
                "bytes" | "bytearray" | "memoryview[Any]" => 4,
                _ => 5,
            },
            Atom::Seq(_) => 6,
            Atom::Tuple(_) => 7,
            Atom::Map(..) => 8,
            Atom::Null => 9,
        }
    }

    fn covers(big: &Atom, small: &Atom) -> bool {
        match (big, small) {
            _ if big == small => true,
            (Atom::Seq(b), Atom::Seq(s)) => subset(s, b),
            (Atom::Seq(b), Atom::Tuple(items)) => items.iter().all(|i| subset(i, b)),
            (Atom::Map(bk, bv), Atom::Map(sk, sv)) => subset(sk, bk) && subset(sv, bv),
            _ => false,
        }
    }

    fn subset(small: &[Atom], big: &[Atom]) -> bool {
        small.iter().all(|s| big.iter().any(|b| covers(b, s)))
    }

    /// Dedupe, drop members another member already covers, and sort.
    fn normalize(atoms: Vec<Atom>) -> Vec<Atom> {
        let mut unique: Vec<Atom> = Vec::new();
        for atom in atoms {
            if !unique.contains(&atom) {
                unique.push(atom);
            }
        }
        let kept: Vec<Atom> = unique
            .iter()
            .filter(|a| !unique.iter().any(|b| b != *a && covers(b, a)))
            .cloned()
            .collect();
        let mut keyed: Vec<(u8, String, Atom)> = kept
            .into_iter()
            .map(|a| (rank(&a), render_atom(&a, Mode::Param), a))
            .collect();
        keyed.sort_by(|x, y| (x.0, &x.1).cmp(&(y.0, &y.1)));
        keyed.into_iter().map(|(_, _, a)| a).collect()
    }

    /// A parameter takes NumPy values wherever it takes numbers, as AGENTS.md rule 6 says:
    /// `int` is `SupportsIndex`, `float` is `SupportsFloat`, and a list of numbers also
    /// takes an ndarray. A union that takes floats drops `int`, which `SupportsFloat` covers.
    fn render(union: &[Atom], mode: Mode) -> String {
        let any = union.iter().any(|a| *a == Atom::Name("Any".into()));
        if union.is_empty() || any {
            return "Any".into();
        }
        let float = Atom::Name("float".into());
        let int = Atom::Name("int".into());
        let mut parts: Vec<String> = union
            .iter()
            .filter(|a| !(mode == Mode::Param && **a == int && union.contains(&float)))
            .map(|a| render_atom(a, mode))
            .collect();
        let numbers = union.iter().any(|a| matches!(a, Atom::Seq(_)) && numeric(a));
        let array = parts.iter().any(|p| p.starts_with("npt.NDArray"));
        if mode == Mode::Param && numbers && !array {
            let before_none = parts.iter().position(|p| p == "None").unwrap_or(parts.len());
            parts.insert(before_none, "npt.NDArray[Any]".into());
        }
        parts.join(" | ")
    }

    /// Whether the atom is a number or a list of numbers, at any depth.
    fn numeric(atom: &Atom) -> bool {
        match atom {
            Atom::Name(n) => n == "int" || n == "float",
            Atom::Seq(inner) => !inner.is_empty() && inner.iter().all(numeric),
            _ => false,
        }
    }

    fn render_atom(atom: &Atom, mode: Mode) -> String {
        match (atom, mode) {
            (Atom::Name(n), Mode::Param) if n == "int" => "SupportsIndex".into(),
            (Atom::Name(n), Mode::Param) if n == "float" => "SupportsFloat".into(),
            (Atom::Name(n), _) => n.clone(),
            (Atom::Seq(inner), Mode::Param) => format!("Sequence[{}]", render(inner, mode)),
            (Atom::Seq(inner), Mode::Return) => format!("list[{}]", render(inner, mode)),
            (Atom::Tuple(items), _) => format!(
                "tuple[{}]",
                items
                    .iter()
                    .map(|i| render(i, mode))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            // pyo3 extracts a HashMap only from a dict. A parameter says Mapping, whose values
            // are covariant, so a caller's `dict[str, int]` fits `Mapping[str, SupportsIndex]`.
            (Atom::Map(k, v), Mode::Param) => {
                format!("Mapping[{}, {}]", render(k, mode), render(v, mode))
            }
            (Atom::Map(k, v), Mode::Return) => {
                format!("dict[{}, {}]", render(k, mode), render(v, mode))
            }
            (Atom::Null, _) => "None".into(),
        }
    }

    /// The atoms of a hint's text, split on its top-level `|`, with `Sequence[...]` read
    /// as a list, so an annotated parameter renders its numbers as a typed one does.
    fn parse_hint(text: &str) -> Vec<Atom> {
        let mut parts = Vec::new();
        let (mut depth, mut start) = (0usize, 0usize);
        for (index, letter) in text.char_indices() {
            match letter {
                '[' => depth += 1,
                ']' => depth = depth.saturating_sub(1),
                '|' if depth == 0 => {
                    parts.push(&text[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
        parts.push(&text[start..]);
        parts
            .into_iter()
            .map(str::trim)
            .map(|part| {
                match part
                    .strip_prefix("Sequence[")
                    .and_then(|rest| rest.strip_suffix(']'))
                {
                    Some(inner) => Atom::Seq(parse_hint(inner)),
                    None => Atom::Name(part.to_string()),
                }
            })
            .collect()
    }

    fn name(n: &str) -> Vec<Atom> {
        vec![Atom::Name(n.to_string())]
    }

    /// Python types a value of the Rust type `ty` can be, in this build.
    fn expand(ty: &Type, mode: Mode, scope: &Scope, self_name: &str, ctx: &Ctx) -> Vec<Atom> {
        let ty = strip_refs(ty);
        let atoms = match ty {
            Type::ImplTrait(it) => {
                let bounds: Vec<syn::TypeParamBound> = it.bounds.iter().cloned().collect();
                expand_bounds(&bounds, mode, scope, self_name, ctx)
            }
            Type::TraitObject(to) => {
                let bounds: Vec<syn::TypeParamBound> = to.bounds.iter().cloned().collect();
                expand_bounds(&bounds, mode, scope, self_name, ctx)
            }
            Type::Tuple(t) if t.elems.is_empty() => vec![Atom::Null],
            Type::Tuple(t) => {
                let items: Vec<Vec<Atom>> = t
                    .elems
                    .iter()
                    .map(|e| expand(e, mode, scope, self_name, ctx))
                    .collect();
                if items.iter().any(Vec::is_empty) {
                    Vec::new()
                } else {
                    vec![Atom::Tuple(items)]
                }
            }
            Type::Array(a) => sequence_of(&a.elem, mode, scope, self_name, ctx),
            Type::Slice(s) => sequence_of(&s.elem, mode, scope, self_name, ctx),
            Type::Path(p) => expand_path(ty, p, mode, scope, self_name, ctx),
            _ => Vec::new(),
        };
        normalize(atoms)
    }

    fn expand_path(
        ty: &Type,
        p: &syn::TypePath,
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        let ident = last_ident(&p.path);
        if p.qself.is_none()
            && p.path.segments.len() == 1
            && let Some(bounds) = scope.bounds.get(&ident)
        {
            return expand_bounds(bounds, mode, scope, self_name, ctx);
        }
        let args = type_args(ty);
        let inner = |i: usize| -> Vec<Atom> {
            args.get(i)
                .map(|t| expand(t, mode, scope, self_name, ctx))
                .unwrap_or_default()
        };
        match ident.as_str() {
            "Self" => name(self_name),
            "bool" => name("bool"),
            "f32" | "f64" => name("float"),
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" => name("int"),
            "String" | "str" | "char" | "Cow" | "PathBuf" | "Path" | "OsStr" | "OsString" => {
                name("str")
            }
            "Option" => {
                let mut atoms = inner(0);
                if atoms.is_empty() {
                    atoms = name("Any");
                }
                atoms.push(Atom::Null);
                atoms
            }
            "Vec" | "VecDeque" | "HashSet" | "BTreeSet" => match args.first() {
                Some(elem) => sequence_of(elem, mode, scope, self_name, ctx),
                None => Vec::new(),
            },
            "HashMap" | "BTreeMap" => {
                let (k, v) = (inner(0), inner(1));
                let k = if k.is_empty() { name("Any") } else { k };
                let v = if v.is_empty() { name("Any") } else { v };
                vec![Atom::Map(k, v)]
            }
            "Result" | "PyResult" | "Py" | "Bound" | "Borrowed" | "PyRef" | "PyRefMut" | "Box"
            | "Arc" | "Rc" => inner(0),
            "PyAny" | "PyObject" => name("Any"),
            "PyDict" => name("dict[str, Any]"),
            "PyList" => name("list[Any]"),
            "PyTuple" => name("tuple[Any, ...]"),
            "PyBytes" => name("bytes"),
            "PyByteArray" => name("bytearray"),
            "PyMemoryView" => name("memoryview[Any]"),
            "PyString" => name("str"),
            "PyType" => name("type"),
            "PyFloat" => name("float"),
            "PyInt" | "PyLong" => name("int"),
            "PyBool" => name("bool"),
            "PyArray1" | "PyArray2" | "PyArray3" | "PyArrayDyn" | "PyReadonlyArray1"
            | "PyReadonlyArray2" | "PyReadonlyArray3" | "PyReadonlyArrayDyn" => {
                let dtype = match args.first().map(|t| type_ident(t)).as_deref() {
                    Some("u8") => "numpy.uint8",
                    Some("f32") => "numpy.float32",
                    Some("f64") => "numpy.float64",
                    Some("u32") => "numpy.uint32",
                    Some("i32") => "numpy.int32",
                    _ => "Any",
                };
                name(&format!("npt.NDArray[{dtype}]"))
            }
            _ => expand_named(&ident, mode, scope, self_name, ctx),
        }
    }

    /// A crate type: its pyclass, the pyclass that stands in for it, or the
    /// variants of a `derive(FromPyObject)` / `derive(IntoPyObject)` type.
    fn expand_named(
        ident: &str,
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        if ctx.stub_names.contains_key(ident) {
            return class_hint(ident, mode, ctx);
        }
        let def = ctx.model.type_named(ident, Build::Python);
        let derive = match mode {
            Mode::Param => "FromPyObject",
            Mode::Return => "IntoPyObject",
        };
        let derives = def.is_some_and(|def| {
            reflect::metas(&def.attrs, Build::Python).iter().any(|m| {
                reflect::meta_ident(m) == "derive" && reflect::meta_args(m).contains(derive)
            })
        });
        let Some(def) = def.filter(|_| derives) else {
            // A core type Python sees through a pyclass of the same name.
            return match ctx.faces.get(ident) {
                Some(rust) => class_hint(rust, mode, ctx),
                None => Vec::new(),
            };
        };
        if ctx.expanding.borrow().iter().any(|e| e == ident) {
            return Vec::new();
        }
        ctx.expanding.borrow_mut().push(ident.to_string());
        let atoms = match def.kind {
            TypeKind::Enum => def
                .variants
                .iter()
                .filter(|v| v.cfg.holds(Build::Python))
                .flat_map(|v| match v.fields.as_slice() {
                    [only] => annotated(
                        expand(only, mode, scope, self_name, ctx),
                        &v.attrs,
                        mode,
                        ctx,
                    ),
                    [] => Vec::new(),
                    many => {
                        let items: Vec<Vec<Atom>> = many
                            .iter()
                            .map(|f| expand(f, mode, scope, self_name, ctx))
                            .collect();
                        if items.iter().any(Vec::is_empty) {
                            Vec::new()
                        } else {
                            vec![Atom::Tuple(items)]
                        }
                    }
                })
                .collect(),
            // A struct with named fields reads from a mapping when it asks
            // for items and from attributes otherwise, which no hint states.
            TypeKind::Struct => {
                let from_items = reflect::metas(&def.attrs, Build::Python)
                    .iter()
                    .any(|m| reflect::meta_args(m).contains("from_item_all"));
                if mode == Mode::Return || from_items {
                    vec![Atom::Map(name("str"), name("Any"))]
                } else {
                    Vec::new()
                }
            }
        };
        ctx.expanding.borrow_mut().pop();
        atoms
    }

    /// A variant's `#[pyo3(annotation = "...")]` when it narrows a value or a
    /// container its Rust type leaves untyped, such as `list[int]` for a `Bound<PyList>`,
    /// or `int` and `Sequence[float]` for a `Bound<PyAny>` that the binding checks itself.
    /// Every name in it must be a builtin or a class Python can hold. A `dict[...]`
    /// annotation narrows only a return: `dict` is invariant, so a parameter hinted
    /// `dict[str, bool | None]` would turn away a caller's `dict[str, bool]`.
    fn annotated(atoms: Vec<Atom>, attrs: &[Attribute], mode: Mode, ctx: &Ctx) -> Vec<Atom> {
        const BUILTINS: &[&str] = &[
            "int", "float", "str", "bytes", "bool", "None", "list", "dict", "tuple", "Sequence",
        ];
        let Some(annotation) = pyo3_args(attrs).and_then(|a| arg_value(&a, "annotation")) else {
            return atoms;
        };
        let narrows = |generic: &str, kind: &str| {
            atoms == [Atom::Name(generic.into())] && annotation.starts_with(kind)
        };
        let dict = mode == Mode::Return && narrows("dict[str, Any]", "dict[");
        let sequence = atoms == [Atom::Seq(name("Any"))] && annotation.starts_with("Sequence[");
        let value = atoms == name("Any") && !annotation.starts_with("dict[");
        if !(narrows("list[Any]", "list[") || dict || sequence || value) {
            return atoms;
        }
        let mut text = annotation.clone();
        let names: BTreeSet<&str> = annotation
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .collect();
        for word in names {
            if BUILTINS.contains(&word) {
                continue;
            }
            let rust = ctx
                .faces
                .get(word)
                .cloned()
                .or_else(|| ctx.stub_names.contains_key(word).then(|| word.to_string()));
            let Some(Atom::Name(stub)) = rust.and_then(|r| class_hint(&r, mode, ctx).pop()) else {
                return atoms;
            };
            text = reflect::replace_qualified(&text, word, &stub);
        }
        match mode {
            Mode::Param => parse_hint(&text),
            Mode::Return => vec![Atom::Name(text)],
        }
    }

    /// The stub name of the pyclass `rust`, unless it is a parameter Python
    /// could never pass.
    fn class_hint(rust: &str, mode: Mode, ctx: &Ctx) -> Vec<Atom> {
        let hidden = mode == Mode::Param
            && ctx
                .obtainable
                .borrow()
                .as_ref()
                .is_some_and(|known| !known.contains(rust));
        if hidden {
            return Vec::new();
        }
        ctx.referenced.borrow_mut().insert(rust.to_string());
        name(&ctx.stub_names[rust])
    }

    /// pyo3 returns `Vec<u8>` and `[u8; N]` as `bytes`, and accepts them from
    /// `bytes` or any sequence of ints. Other element types round-trip as lists.
    fn sequence_of(
        elem: &Type,
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        if is_u8(elem) {
            return match mode {
                Mode::Param => vec![Atom::Name("bytes".into()), Atom::Seq(name("int"))],
                Mode::Return => name("bytes"),
            };
        }
        let inner = expand(elem, mode, scope, self_name, ctx);
        if inner.is_empty() {
            Vec::new()
        } else {
            vec![Atom::Seq(inner)]
        }
    }

    /// Types that satisfy every non-marker bound.
    fn expand_bounds(
        bounds: &[syn::TypeParamBound],
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        let mut sets: Vec<Vec<Atom>> = Vec::new();
        for bound in bounds {
            let syn::TypeParamBound::Trait(tb) = bound else {
                continue;
            };
            let Some(segment) = tb.path.segments.last() else {
                continue;
            };
            let trait_name = segment.ident.to_string();
            if MARKER_TRAITS.contains(&trait_name.as_str()) {
                continue;
            }
            let (arg, assoc) = bound_args(segment);
            let set = match (trait_name.as_str(), arg) {
                ("Into", Some(target)) => into_sources(target, mode, scope, self_name, ctx),
                ("TryInto", Some(target)) => {
                    let infallible_too = !assoc.iter().any(|(k, _)| k == "Error");
                    try_sources(target, infallible_too, mode, scope, self_name, ctx)
                }
                ("IntoIterator" | "Iterator", _) => match assoc.iter().find(|(k, _)| k == "Item") {
                    Some((_, item)) => sequence_of(item, mode, scope, self_name, ctx),
                    None => Vec::new(),
                },
                ("AsRef", Some(target)) => expand(target, mode, scope, self_name, ctx),
                ("From", Some(source)) => from_targets(source, mode, self_name, ctx),
                _ => implementors(&trait_name, mode, self_name, ctx),
            };
            sets.push(set);
        }
        let mut sets = sets.into_iter();
        let Some(first) = sets.next() else {
            return Vec::new();
        };
        sets.fold(first, |acc, set| {
            acc.into_iter().filter(|a| set.contains(a)).collect()
        })
    }

    /// The first type argument of a bound and its associated type bindings.
    fn bound_args(segment: &syn::PathSegment) -> (Option<&Type>, Vec<(String, &Type)>) {
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            return (None, Vec::new());
        };
        let first = args.args.iter().find_map(|a| match a {
            GenericArgument::Type(t) => Some(t),
            _ => None,
        });
        let assoc = args
            .args
            .iter()
            .filter_map(|a| match a {
                GenericArgument::AssocType(at) => Some((at.ident.to_string(), &at.ty)),
                _ => None,
            })
            .collect();
        (first, assoc)
    }

    /// `T: Into<X>`: `X` itself and every `From` source of `X`.
    fn into_sources(
        target: &Type,
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        let mut atoms = expand(target, mode, scope, self_name, ctx);
        for (imp, source) in ctx
            .model
            .conversions_into(&type_ident(target), "From", Build::Python)
        {
            let impl_scope = scope_of(&[&imp.generics]);
            atoms.extend(expand(source, mode, &impl_scope, self_name, ctx));
        }
        atoms
    }

    /// `T: TryInto<X>`: every `TryFrom` source of `X`, plus the `From`
    /// sources when the bound accepts the infallible conversion too.
    fn try_sources(
        target: &Type,
        infallible_too: bool,
        mode: Mode,
        scope: &Scope,
        self_name: &str,
        ctx: &Ctx,
    ) -> Vec<Atom> {
        let mut atoms = Vec::new();
        for (imp, source) in
            ctx.model
                .conversions_into(&type_ident(target), "TryFrom", Build::Python)
        {
            let impl_scope = scope_of(&[&imp.generics]);
            atoms.extend(expand(source, mode, &impl_scope, self_name, ctx));
        }
        if infallible_too {
            atoms.extend(into_sources(target, mode, scope, self_name, ctx));
        }
        atoms
    }

    /// `T: From<S>`: every type with an `impl From<S>`.
    fn from_targets(source: &Type, mode: Mode, self_name: &str, ctx: &Ctx) -> Vec<Atom> {
        let source = type_ident(source);
        ctx.model
            .trait_impls("From", Build::Python)
            .filter(|imp| imp.trait_arg().map(type_ident).as_deref() == Some(source.as_str()))
            .flat_map(|imp| {
                expand(
                    &imp.self_ty,
                    mode,
                    &scope_of(&[&imp.generics]),
                    self_name,
                    ctx,
                )
            })
            .collect()
    }

    /// Every implementor of `trait_name`. A trait implemented for collections
    /// of itself (`Vec<Box<dyn Renderable>>`) is expanded one level deep, so
    /// a list of renderables is a hint and a list of lists is not.
    fn implementors(trait_name: &str, mode: Mode, self_name: &str, ctx: &Ctx) -> Vec<Atom> {
        let depth = ctx
            .expanding
            .borrow()
            .iter()
            .filter(|e| *e == trait_name)
            .count();
        if depth >= 2 {
            return Vec::new();
        }
        ctx.expanding.borrow_mut().push(trait_name.to_string());
        let mut atoms = Vec::new();
        for imp in ctx.model.trait_impls(trait_name, Build::Python) {
            if depth == 1 && is_collection(&imp.self_ty) {
                continue;
            }
            let scope = scope_of(&[&imp.generics]);
            atoms.extend(expand(&imp.self_ty, mode, &scope, self_name, ctx));
        }
        ctx.expanding.borrow_mut().pop();
        atoms
    }

    fn is_collection(ty: &Type) -> bool {
        match strip_refs(ty) {
            Type::Slice(_) | Type::Array(_) => true,
            other => matches!(type_ident(other).as_str(), "Vec" | "VecDeque"),
        }
    }

    /// Hint for a binding value, and the Rust source a shared alias would be
    /// named after. Untyped values take the canonical method's type.
    fn value_hint(
        method: &Method,
        param: Option<(&str, &Type)>,
        mode: Mode,
        self_name: &str,
        ctx: &Ctx,
    ) -> (Vec<Atom>, Option<String>) {
        let binding_ty = match param {
            Some((_, ty)) => Some(ty),
            None => method.ret.as_ref(),
        };
        let Some(binding_ty) = binding_ty else {
            return (vec![Atom::Null], None);
        };
        if !is_untyped(binding_ty) {
            let atoms = expand(binding_ty, mode, &Scope::default(), self_name, ctx);
            if atoms.is_empty() {
                ctx.unknown
                    .borrow_mut()
                    .insert(binding_ty.to_token_stream().to_string());
            }
            let source = source_name(binding_ty, &Scope::default(), ctx);
            return (atoms, source);
        }
        let Some((canonical, scope, owner)) = canonical_value(method, param, ctx) else {
            if method.lsp_doc.is_some() {
                let owner = ctx.stub_names.get(&method.owner).map_or("", String::as_str);
                let value = param.map_or("return", |(name, _)| name);
                ctx.unmatched
                    .borrow_mut()
                    .insert(format!("{owner}.{}({value})", method.py_name));
            }
            return (Vec::new(), None);
        };
        let owner_name = ctx
            .stub_names
            .get(&owner)
            .or_else(|| ctx.faces.get(&owner).map(|rust| &ctx.stub_names[rust]))
            .map(String::as_str)
            .unwrap_or(self_name);
        let atoms = expand(&canonical, mode, &scope, owner_name, ctx);
        let source = source_name(&canonical, &scope, ctx);
        (atoms, source)
    }

    /// The canonical method's type for a binding parameter (or its return
    /// type when `param` is `None`), with the generics in scope and the
    /// canonical owner's Rust name.
    fn canonical_value(
        method: &Method,
        param: Option<(&str, &Type)>,
        ctx: &Ctx,
    ) -> Option<(Type, Scope, String)> {
        // The method documented on the same page, else the method with the
        // same name on the type this binding wraps.
        let page = method
            .lsp_doc
            .as_deref()
            .map(|p| canonical_page(ctx.root, p));
        let wrapped: Vec<&str> = std::iter::once(method.owner.as_str())
            .chain(
                ctx.faces
                    .iter()
                    .filter(|(_, rust)| **rust == method.owner)
                    .map(|(py, _)| py.as_str()),
            )
            .collect();
        let rust_name = method
            .rust_name
            .strip_suffix("_py")
            .unwrap_or(&method.rust_name);
        let documented = ctx
            .model
            .plain_methods(Build::Python)
            .find(|(_, f)| page.is_some() && reflect::lsp_doc(&f.attrs) == page);
        let (imp, f) = documented.or_else(|| {
            ctx.model.plain_methods(Build::Python).find(|(imp, f)| {
                wrapped.contains(&imp.self_ident.as_str()) && f.sig.ident == rust_name
            })
        })?;
        let scope = scope_of(&[&imp.generics, &f.sig.generics]);
        let owner = imp.self_ident.clone();
        let Some((param_name, _)) = param else {
            return match &f.sig.output {
                ReturnType::Type(_, ty) => Some(((**ty).clone(), scope, owner)),
                ReturnType::Default => None,
            };
        };
        let typed: Vec<(String, &Type)> = f
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(t) => match t.pat.as_ref() {
                    Pat::Ident(p) => Some((p.ident.to_string(), t.ty.as_ref())),
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect();
        // Match by name; parameters renamed for Python match by position.
        let by_name = typed.iter().find(|(n, _)| n == param_name);
        let by_position = (typed.len() == method.params.len())
            .then(|| method.params.iter().position(|(n, _)| n == param_name))
            .flatten()
            .map(|i| &typed[i]);
        let (_, ty) = by_name.or(by_position)?;
        Some(((*ty).clone(), scope, owner))
    }

    /// The Rust name a union is known by: the trait of `impl Trait`, the `X`
    /// of `Into<X>`, or a `derive(FromPyObject)` type.
    fn source_name(ty: &Type, scope: &Scope, ctx: &Ctx) -> Option<String> {
        let bounds: Vec<syn::TypeParamBound> = match strip_refs(ty) {
            Type::ImplTrait(it) => it.bounds.iter().cloned().collect(),
            Type::TraitObject(to) => to.bounds.iter().cloned().collect(),
            Type::Path(p) => {
                let ident = last_ident(&p.path);
                let generic = p.path.segments.len() == 1 && scope.bounds.contains_key(&ident);
                match ident.as_str() {
                    "Option" => {
                        return type_args(strip_refs(ty))
                            .first()
                            .and_then(|t| source_name(t, scope, ctx));
                    }
                    _ if generic => scope.bounds[&ident].clone(),
                    _ if !ctx.stub_names.contains_key(&ident)
                        && ctx.model.type_named(&ident, Build::Python).is_some() =>
                    {
                        return Some(ident);
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        let named: Vec<String> = bounds
            .iter()
            .filter_map(|b| match b {
                syn::TypeParamBound::Trait(tb) => tb.path.segments.last(),
                _ => None,
            })
            .filter(|s| !MARKER_TRAITS.contains(&s.ident.to_string().as_str()))
            .map(|s| match (s.ident.to_string().as_str(), bound_args(s).0) {
                ("Into" | "TryInto", Some(target)) => type_ident(target),
                (other, _) => other.to_string(),
            })
            .collect();
        match named.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }

    fn placeholder(name: &str) -> String {
        format!("\u{ab}{name}\u{bb}")
    }

    /// The text a parameter hint renders as: a placeholder when the union is
    /// big enough to share, else the union itself.
    fn param_text(atoms: &[Atom], source: Option<String>, ctx: &Ctx) -> String {
        let optional = atoms.contains(&Atom::Null);
        let core: Vec<Atom> = atoms
            .iter()
            .filter(|a| **a != Atom::Null)
            .cloned()
            .collect();
        let Some(source) = source.filter(|_| core.len() >= 3) else {
            return render(atoms, Mode::Param);
        };
        let text = render(&core, Mode::Param);
        let mut aliases = ctx.aliases.borrow_mut();
        if aliases
            .get(&source)
            .is_some_and(|existing| *existing != text)
        {
            return render(atoms, Mode::Param);
        }
        aliases.insert(source.clone(), text);
        let token = placeholder(&source);
        if optional {
            format!("{token} | None")
        } else {
            token
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
    ) -> String {
        let stub_name = ctx.stub_names[rust].as_str();
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
            let hint = render(
                &expand(&field.ty, Mode::Return, &Scope::default(), stub_name, ctx),
                Mode::Return,
            );
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
            body.push_str(&render_method(root, stub_name, method, ctx, 4));
        }
        if body.is_empty() {
            out.truncate(out.len() - 1);
            out.push_str(" ...\n");
        } else {
            out.push_str(&body);
        }
        out
    }

    /// `self_name` is the owner's stub name, for `Self`. Module functions pass
    /// an empty string.
    fn render_method(
        root: &Path,
        self_name: &str,
        method: &Method,
        ctx: &Ctx,
        indent: usize,
    ) -> String {
        let pad = " ".repeat(indent);

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
                let (atoms, source) = value_hint(
                    method,
                    Some((name.as_str(), ty)),
                    Mode::Param,
                    self_name,
                    ctx,
                );
                (py_param_name(name), param_text(&atoms, source, ctx))
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
            render(
                &value_hint(method, None, Mode::Return, self_name, ctx).0,
                Mode::Return,
            )
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
