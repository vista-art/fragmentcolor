mod kotlin {
    //! Kotlin transpilation — second phase.
    //!
    //! `convert::to_kotlin` runs the per-line emitter with `Lang::Kotlin`,
    //! producing a JS-shaped intermediate. This module finishes the job
    //! with the bulk-text swaps that don't fit a per-line model.
    //!
    //! Kotlin differs from Swift in:
    //!   - `null` stays as `null` (Kotlin keyword).
    //!   - `const` → `val` (not `let`).
    //!   - Async methods are `suspend` functions; callers drop `await`
    //!     entirely and run inside `runBlocking { ... }` or another
    //!     coroutine scope (the healthcheck wrapper handles this).
    //!   - Import becomes `import org.fragmentcolor.*`.
    //!   - Trailing semicolons are optional and we strip them, except in
    //!     the shader source inside triple-quoted strings.
    //!
    //! Literal suffixes (`0.5f`, `1u`) and `arrayOf` → `listOf` follow the parameter types
    //! of the uniffi exports for Android and of the hand-written `*Extensions.kt` files.
    //! So does a Rust tuple argument, which spreads over those parameters,
    //! and an array literal at a record parameter, which fills the record's fields in order.
    //! An `operator fun T.Companion.invoke(...)` in an extension file is a factory
    //! that a `T::new(...)` call reaches.
    //!
    //! Enum entries follow the flat uniffi enums.

    use std::collections::{BTreeMap, BTreeSet};

    use heck::{ToLowerCamelCase, ToShoutySnakeCase};

    use crate::reflect::{Build, CallKind, Reflection, SliceGetter, StandIn, replace_qualified};
    use crate::source_text::{
        Nesting, arg_spans, closing, code_only, drop_semicolon_before_comment, extension_files,
        is_ident_char, read_ident, skip_space, split_args, top_level_words,
    };

    /// The extension functions written by hand on top of uniffi's Kotlin.
    const EXTENSIONS_DIR: &str = "platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor";

    /// A Kotlin parameter type, reduced to what the call-site rewrites need.
    #[derive(Clone, PartialEq, Eq)]
    enum KType {
        /// `Float`. A Kotlin `0.5` is a `Double` and doesn't convert.
        Float,
        /// `UInt`, `ULong`, `UShort` or `UByte`, by name.
        Unsigned(String),
        /// `Int`, which an integer literal fits as written.
        Int,
        /// Any other number: `Double`, `Long`, `Short` or `Byte`.
        Number,
        List(Box<KType>),
        /// `Array<T>`, which `arrayOf(...)` already builds.
        Array,
        /// `FloatArray`, `ByteArray` and the other primitive arrays, by name.
        PrimitiveArray(String),
        /// Any other type, by name.
        Named(String),
    }

    /// A function Kotlin code can call, from uniffi or an extension file.
    struct Callable {
        /// The type it belongs to. `None` for a top-level function.
        owner: Option<String>,
        /// How a call starts: `.render(`, `Vertex(`, `Vertex.new(` or `ShaderFetch(`.
        needle: String,
        params: Vec<KType>,
        /// How many trailing parameters have a default.
        defaults: usize,
        /// How many trailing parameters are a Rust `Option`,
        /// which a call to a uniffi export still passes, as `null`.
        nullable: usize,
        /// Whether uniffi generates it.
        generated: bool,
    }

    impl Callable {
        /// Whether a call with `argc` arguments can resolve to it.
        fn takes(&self, argc: usize) -> bool {
            (self.params.len() - self.defaults..=self.params.len()).contains(&argc)
        }
    }

    /// What a literal argument needs at one position of a call.
    struct Slot {
        /// A number needs the `f` suffix.
        float: bool,
        /// Every receiver takes an `Int` here, so an integer literal stays as written.
        int: bool,
        /// An integer needs the `u` suffix.
        /// Holds the Kotlin type a value converts to
        /// when an array spreads into separate arguments.
        unsigned: Option<String>,
        /// How `arrayOf(...)` becomes a `List` here.
        list: Option<Elems>,
        /// The type an `arrayOf(numbers)` here builds through its constructor.
        wrap: Option<String>,
        /// The record an `arrayOf(numbers)` here fills by position, such as `Size`.
        record: Option<String>,
        /// Every receiver takes a `FloatArray` here, which `arrayOf(numbers)` becomes.
        float_array: bool,
    }

    /// How a list's elements change when `arrayOf` becomes `listOf`.
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Elems {
        AsIs,
        Float,
        /// Each element is itself a list of `Float`s, such as a vertex position.
        Rows,
        Unsigned,
        /// Each `arrayOf(numbers)` element builds this type.
        Wrap(String),
    }

    impl Elems {
        /// Which of several shapes every receiver takes is written:
        /// floats first, then plain numbers, which need no `u` suffix,
        /// and a constructor around each element last.
        fn preference(&self) -> u8 {
            match self {
                Elems::Float => 0,
                Elems::Rows => 1,
                Elems::AsIs => 2,
                Elems::Unsigned => 3,
                Elems::Wrap(_) => 4,
            }
        }
    }

    /// Slots by needle, then by argument count.
    type CallPlans = BTreeMap<String, BTreeMap<usize, Vec<Slot>>>;

    /// One field of a uniffi record, as Kotlin names it.
    struct Field {
        name: String,
        ty: KType,
        /// Whether it holds a Rust `Option`.
        optional: bool,
    }

    /// The fields of each record an array literal fills by position, by record name.
    type Records = BTreeMap<String, Vec<Field>>;

    /// A call position whose receivers want opposite rewrites, so the rewrites skip it.
    struct Conflict {
        needle: String,
        argc: usize,
        position: usize,
        /// Which receivers disagree, and on what.
        reason: String,
        /// Whether the skipped rewrite would change an argument.
        changes: fn(&str) -> bool,
    }

    /// API facts the Kotlin rewrites read, derived from the source model.
    struct Facts {
        /// Crate types Kotlin names by the mobile-only type that converts into them.
        stand_ins: Vec<StandIn>,
        calls: CallPlans,
        conflicts: Vec<Conflict>,
        /// How many arguments a tuple of n items spreads into, by needle and then n.
        spreads: BTreeMap<String, BTreeMap<usize, usize>>,
        records: Records,
        /// Kotlin spelling of each flat enum entry,
        /// keyed by its Rust and its Kotlin spelling.
        enums: BTreeMap<String, BTreeMap<String, String>>,
        slice_getters: Vec<SliceGetter>,
    }

    static FACTS: std::sync::OnceLock<Facts> = std::sync::OnceLock::new();

    /// Derive the Kotlin rewrites' API facts. Runs once before any example converts.
    pub fn install(model: &Reflection) {
        let mut callables = Vec::new();
        for f in model.uniffi_fns(Build::Android) {
            let params: Vec<KType> = f.params.iter().map(uniffi_type).collect();
            let required = &f.params[..f.params.len() - f.defaults];
            let nullable = required.iter().rev().take_while(|t| is_option(t)).count();
            let needles = match f.kind {
                CallKind::Init => vec![format!("{}(", f.owner), format!("{}.new(", f.owner)],
                CallKind::Static | CallKind::Method => vec![format!(".{}(", f.name)],
            };
            for needle in needles {
                callables.push(Callable {
                    owner: Some(f.owner.clone()),
                    needle,
                    params: params.clone(),
                    defaults: f.defaults,
                    nullable,
                    generated: true,
                });
            }
        }
        callables.extend(extension_callables());
        let records = positional_records(model);
        let mut conflicts = Vec::new();
        let calls = plan_calls(&callables, &records, &mut conflicts);
        let _ = FACTS.set(Facts {
            stand_ins: model.stand_ins(Build::Android),
            calls,
            conflicts,
            spreads: plan_spreads(&callables),
            records,
            enums: flat_enum_spellings(model),
            slice_getters: model.slice_getters(Build::Android),
        });
    }

    fn facts() -> &'static Facts {
        FACTS
            .get()
            .unwrap_or_else(|| panic!("kotlin::install must run before examples convert"))
    }

    /// Each flat enum's Kotlin entries, keyed by their Rust and their Kotlin spelling.
    fn flat_enum_spellings(model: &Reflection) -> BTreeMap<String, BTreeMap<String, String>> {
        model
            .flat_enums(Build::Android)
            .into_iter()
            .map(|e| {
                let spellings = e
                    .variants
                    .iter()
                    .flat_map(|v| {
                        let entry = v.to_shouty_snake_case();
                        [(v.clone(), entry.clone()), (entry.clone(), entry)]
                    })
                    .collect();
                (e.ident, spellings)
            })
            .collect()
    }

    /// The Kotlin type uniffi generates for a Rust parameter type.
    fn uniffi_type(ty: &syn::Type) -> KType {
        let segment = match ty {
            syn::Type::Reference(r) => return uniffi_type(&r.elem),
            syn::Type::Path(p) => p.path.segments.last(),
            _ => None,
        };
        let Some(segment) = segment else {
            return KType::Named(String::new());
        };
        let inner = match &segment.arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| match arg {
                syn::GenericArgument::Type(t) => Some(t),
                _ => None,
            }),
            _ => None,
        };
        let ident = segment.ident.to_string();
        match (ident.as_str(), inner) {
            ("f32", _) => KType::Float,
            ("u8", _) => KType::Unsigned("UByte".into()),
            ("u16", _) => KType::Unsigned("UShort".into()),
            ("u32", _) => KType::Unsigned("UInt".into()),
            ("u64", _) => KType::Unsigned("ULong".into()),
            ("i32", _) => KType::Int,
            ("f64" | "i8" | "i16" | "i64", _) => KType::Number,
            ("Option" | "Arc" | "Box", Some(t)) => uniffi_type(t),
            // uniffi passes bytes as a `ByteArray`.
            ("Vec", Some(t)) if uniffi_type(t) == KType::Unsigned("UByte".into()) => {
                KType::PrimitiveArray("ByteArray".into())
            }
            ("Vec", Some(t)) => KType::List(Box::new(uniffi_type(t))),
            _ => KType::Named(ident),
        }
    }

    /// The type a Kotlin parameter declaration such as `List<Float>` names.
    fn kotlin_type(text: &str) -> KType {
        let text = text.trim().trim_end_matches('?');
        if let Some(element) = text.strip_prefix("List<").and_then(|t| t.strip_suffix('>')) {
            return KType::List(Box::new(kotlin_type(element)));
        }
        match text {
            "Float" => KType::Float,
            "UInt" | "ULong" | "UShort" | "UByte" => KType::Unsigned(text.to_string()),
            "Int" => KType::Int,
            "Double" | "Long" | "Short" | "Byte" => KType::Number,
            _ if text.starts_with("Array<") => KType::Array,
            _ if text.ends_with("Array") => KType::PrimitiveArray(text.to_string()),
            _ => KType::Named(text.to_string()),
        }
    }

    /// Every top-level `fun` in the `*Extensions.kt` files:
    /// `fun T.name(`, `fun T.Companion.name(` or `fun name(`.
    /// An `operator fun T.Companion.invoke(` is a factory, which a call reaches as `T(` or `T.new(`.
    fn extension_callables() -> Vec<Callable> {
        let mut callables = Vec::new();
        for path in extension_files(EXTENSIONS_DIR, "Extensions.kt") {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("kotlin: cannot read {}: {e}", path.display()));
            let code = code_only(&text);
            for (keyword, at) in top_level_words(&code, &["fun"]) {
                let start = skip_space(&code, at + keyword.len());
                let Some(open) = code[start..].find('(').map(|p| start + p) else {
                    continue;
                };
                let Some(close) = closing(&code, open) else {
                    continue;
                };
                let path: Vec<&str> = code[start..open].trim().split('.').collect();
                let line = code[..at].rfind('\n').map_or(0, |p| p + 1);
                let modifiers: Vec<&str> = code[line..at].split_whitespace().collect();
                if modifiers.iter().any(|w| matches!(*w, "private" | "internal")) {
                    continue;
                }
                let operator = modifiers.contains(&"operator");
                let (owner, needles) = match path.as_slice() {
                    [owner, "Companion", "invoke"] if operator => {
                        (Some(owner.to_string()), vec![format!("{owner}("), format!("{owner}.new(")])
                    }
                    [name] => (None, vec![format!("{name}(")]),
                    [owner, name] | [owner, "Companion", name] => {
                        (Some(owner.to_string()), vec![format!(".{name}(")])
                    }
                    _ => continue,
                };
                let params = kotlin_params(&code[open + 1..close]);
                for needle in needles {
                    callables.push(Callable {
                        owner: owner.clone(),
                        needle,
                        defaults: params.iter().filter(|(_, default)| *default).count(),
                        params: params.iter().map(|(ty, _)| ty.clone()).collect(),
                        nullable: 0,
                        generated: false,
                    });
                }
            }
        }
        callables
    }

    /// The type of each `name: Type [= default]` parameter, and whether it has a default.
    fn kotlin_params(params: &str) -> Vec<(KType, bool)> {
        arg_spans(params, Nesting::Types)
            .into_iter()
            .filter_map(|span| {
                let (_, rest) = params[span].split_once(':')?;
                let (ty, default) = match rest.split_once('=') {
                    Some((ty, _)) => (ty, true),
                    None => (rest, false),
                };
                Some((kotlin_type(ty), default))
            })
            .collect()
    }

    /// Slots for every needle and argument count some overload takes.
    /// Positions where receivers need opposite rewrites land in `conflicts` and get none.
    fn plan_calls(callables: &[Callable], records: &Records, conflicts: &mut Vec<Conflict>) -> CallPlans {
        let wraps = float_list_types(callables);
        let mut by_needle: BTreeMap<&str, Vec<&Callable>> = BTreeMap::new();
        for c in callables {
            by_needle.entry(&c.needle).or_default().push(c);
        }
        let mut plans = CallPlans::new();
        for (needle, overloads) in by_needle {
            let counts: BTreeSet<usize> = overloads
                .iter()
                .flat_map(|c| c.params.len() - c.defaults..=c.params.len())
                .collect();
            for argc in counts {
                let fitting: Vec<&Callable> =
                    overloads.iter().copied().filter(|c| c.takes(argc)).collect();
                let slots = (0..argc)
                    .map(|i| Receivers::new(needle, argc, i, &fitting))
                    .map(|receivers| plan_slot(&receivers, &wraps, records, conflicts))
                    .collect();
                plans.entry(needle.to_string()).or_default().insert(argc, slots);
            }
        }
        plans
    }

    /// Types whose generated constructor takes one float list, such as `Vertex`.
    fn float_list_types(callables: &[Callable]) -> BTreeSet<&str> {
        callables
            .iter()
            .filter_map(|c| {
                let owner = c.owner.as_deref()?;
                let float_list = matches!(c.params.as_slice(), [KType::List(e)] if **e == KType::Float);
                (c.generated && float_list && c.needle == format!("{owner}(")).then_some(owner)
            })
            .collect()
    }

    /// The overloads of a call that take `argc` arguments, seen from the argument at `position`.
    struct Receivers<'a> {
        needle: &'a str,
        argc: usize,
        position: usize,
        overloads: &'a [&'a Callable],
        /// The types each owner takes at `position`.
        by_owner: BTreeMap<&'a str, Vec<&'a KType>>,
    }

    impl<'a> Receivers<'a> {
        fn new(
            needle: &'a str,
            argc: usize,
            position: usize,
            overloads: &'a [&'a Callable],
        ) -> Self {
            let mut by_owner: BTreeMap<&str, Vec<&KType>> = BTreeMap::new();
            for c in overloads {
                let owner = c.owner.as_deref().unwrap_or("a top-level function");
                by_owner.entry(owner).or_default().push(&c.params[position]);
            }
            Receivers {
                needle,
                argc,
                position,
                overloads,
                by_owner,
            }
        }

        fn types(&self) -> impl Iterator<Item = &'a KType> + '_ {
            self.by_owner.values().flatten().copied()
        }

        /// Whether some receiver takes a type that passes `test`.
        fn any(&self, test: fn(&KType) -> bool) -> bool {
            self.types().any(test)
        }

        /// The owners whose types at `position` pass `test`.
        fn owners(&self, test: impl Fn(&[&KType]) -> bool) -> Vec<&'a str> {
            self.by_owner
                .iter()
                .filter(|(_, types)| test(types))
                .map(|(owner, _)| *owner)
                .collect()
        }

        /// A conflict at `position`, explained by `reason`,
        /// for a rewrite that would change the arguments `changes` accepts.
        fn conflict(&self, reason: String, changes: fn(&str) -> bool) -> Conflict {
            Conflict {
                needle: self.needle.to_string(),
                argc: self.argc,
                position: self.position,
                reason,
                changes,
            }
        }

        /// A conflict where `wants` take `what` at `position` and `rejects` don't.
        fn disagreement(
            &self,
            what: &str,
            wants: &[&str],
            rejects: &[&str],
            changes: fn(&str) -> bool,
        ) -> Conflict {
            let (n, wants, rejects) = (self.position + 1, wants.join(", "), rejects.join(", "));
            let reason = format!("it takes {what} at argument {n} on {wants} but not on {rejects}");
            self.conflict(reason, changes)
        }
    }

    /// Whether one of `types` passes `test`.
    fn has(types: &[&KType], test: fn(&KType) -> bool) -> bool {
        types.iter().any(|t| test(t))
    }

    fn is_float(t: &KType) -> bool {
        *t == KType::Float
    }

    fn is_unsigned(t: &KType) -> bool {
        matches!(t, KType::Unsigned(_))
    }

    /// `Double`, `Int` and the other numbers a `Float` literal doesn't fit.
    fn is_other_number(t: &KType) -> bool {
        matches!(t, KType::Int | KType::Number | KType::Unsigned(_))
    }

    /// `Float`, `Double`, `Int` and the other numbers an unsigned literal doesn't fit.
    fn is_signed(t: &KType) -> bool {
        matches!(t, KType::Float | KType::Int | KType::Number)
    }

    fn is_int(t: &KType) -> bool {
        *t == KType::Int
    }

    fn is_list(t: &KType) -> bool {
        matches!(t, KType::List(_))
    }

    fn is_array(t: &KType) -> bool {
        matches!(t, KType::Array | KType::PrimitiveArray(_))
    }

    fn is_float_array(t: &KType) -> bool {
        matches!(t, KType::PrimitiveArray(name) if name == "FloatArray")
    }

    /// `Array<T>`, the only parameter type an `arrayOf(...)` already fits, so it needs no `listOf`.
    fn is_object_array(t: &KType) -> bool {
        *t == KType::Array
    }

    /// What the argument at one position of a call needs.
    fn plan_slot(
        receivers: &Receivers,
        wraps: &BTreeSet<&str>,
        records: &Records,
        conflicts: &mut Vec<Conflict>,
    ) -> Slot {
        Slot {
            float: float_slot(receivers, conflicts),
            int: !receivers.by_owner.is_empty() && receivers.owners(|ts| !has(ts, is_int)).is_empty(),
            unsigned: unsigned_slot(receivers, conflicts),
            list: list_slot(receivers, wraps, conflicts),
            wrap: wrap_slot(receivers, wraps),
            record: record_slot(receivers, records),
            float_array: !receivers.by_owner.is_empty()
                && receivers.owners(|ts| !has(ts, is_float_array)).is_empty(),
        }
    }

    /// Whether a number here needs the `f` suffix.
    /// A receiver that also takes a `Double` here gets a `Float` too,
    /// the type a list here becomes, so one overload takes every argument.
    fn float_slot(receivers: &Receivers, conflicts: &mut Vec<Conflict>) -> bool {
        let wants = receivers.owners(|ts| has(ts, is_float) && !has(ts, is_other_number));
        let rejects = receivers.owners(|ts| has(ts, is_other_number) && !has(ts, is_float));
        if !rejects.is_empty() {
            if !wants.is_empty() {
                let changes = |arg: &str| floatify_if_numeric(arg) != arg;
                conflicts.push(receivers.disagreement("a Float", &wants, &rejects, changes));
            }
            return false;
        }
        receivers.any(is_float)
    }

    /// The unsigned type an integer here converts to,
    /// taken from the overloads uniffi generates.
    fn unsigned_slot(receivers: &Receivers, conflicts: &mut Vec<Conflict>) -> Option<String> {
        let unsigned = receivers
            .overloads
            .iter()
            .filter(|c| c.generated)
            .find_map(|c| match &c.params[receivers.position] {
                KType::Unsigned(name) => Some(name.clone()),
                _ => None,
            })?;
        // An `Int` overload next to every unsigned one reads a plain literal.
        if receivers.owners(|ts| has(ts, is_unsigned) && !has(ts, is_int)).is_empty() {
            return None;
        }
        let rejects = receivers.owners(|ts| has(ts, is_signed) && !has(ts, is_unsigned));
        if !rejects.is_empty() {
            let wants = receivers.owners(|ts| has(ts, is_unsigned));
            let changes = |arg: &str| unsigned_literal(arg) != arg;
            let conflict = receivers.disagreement("an unsigned integer", &wants, &rejects, changes);
            conflicts.push(conflict);
            return None;
        }
        Some(unsigned)
    }

    /// How an `arrayOf(...)` here becomes a `List`.
    fn list_slot(
        receivers: &Receivers,
        wraps: &BTreeSet<&str>,
        conflicts: &mut Vec<Conflict>,
    ) -> Option<Elems> {
        let wants = receivers.owners(|ts| has(ts, is_list) && !has(ts, is_object_array));
        let rejects = receivers.owners(|ts| has(ts, is_array) && !has(ts, is_list));
        let changes = |arg: &str| strip_array(arg).is_some();
        if !wants.is_empty() && !rejects.is_empty() {
            conflicts.push(receivers.disagreement("a List", &wants, &rejects, changes));
            return None;
        }
        if wants.is_empty() || receivers.any(is_object_array) {
            return None;
        }
        // The shapes each receiver takes here, and the most preferred one all of them take.
        let taken: Vec<BTreeSet<Elems>> = receivers
            .by_owner
            .values()
            .map(|types| types.iter().filter_map(|t| list_elems(t, wraps)).collect())
            .filter(|shapes: &BTreeSet<Elems>| !shapes.is_empty())
            .collect();
        let mut candidates: Vec<&Elems> = taken.iter().flatten().collect();
        candidates.sort_by_key(|e| e.preference());
        let shared = candidates.into_iter().find(|e| taken.iter().all(|shapes| shapes.contains(*e)));
        if shared.is_none() {
            let n = receivers.position + 1;
            let reason = format!("it takes lists of different element types at argument {n}");
            conflicts.push(receivers.conflict(reason, changes));
        }
        shared.cloned()
    }

    /// How the elements of a `List` parameter type change.
    fn list_elems(ty: &KType, wraps: &BTreeSet<&str>) -> Option<Elems> {
        let KType::List(element) = ty else {
            return None;
        };
        Some(match &**element {
            KType::Float => Elems::Float,
            KType::List(row) if **row == KType::Float => Elems::Rows,
            KType::Unsigned(_) => Elems::Unsigned,
            KType::Named(name) if wraps.contains(name.as_str()) => Elems::Wrap(name.clone()),
            _ => Elems::AsIs,
        })
    }

    /// The one type an `arrayOf(numbers)` here builds through its constructor,
    /// such as `Vertex`, when no receiver takes a list or an array.
    fn wrap_slot(receivers: &Receivers, wraps: &BTreeSet<&str>) -> Option<String> {
        if receivers.any(is_list) || receivers.any(is_array) {
            return None;
        }
        let built: BTreeSet<&str> = receivers
            .types()
            .filter_map(|t| match t {
                KType::Named(name) if wraps.contains(name.as_str()) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        match built.into_iter().collect::<Vec<_>>().as_slice() {
            [only] => Some(only.to_string()),
            _ => None,
        }
    }

    /// The one record an `arrayOf(numbers)` here fills, such as `Size`,
    /// when every receiver takes that record.
    fn record_slot(receivers: &Receivers, records: &Records) -> Option<String> {
        let names: BTreeSet<&str> = receivers
            .types()
            .map(|t| match t {
                KType::Named(name) if records.contains_key(name) => Some(name.as_str()),
                _ => None,
            })
            .collect::<Option<_>>()?;
        match names.into_iter().collect::<Vec<_>>().as_slice() {
            [only] => Some(only.to_string()),
            _ => None,
        }
    }

    /// Whether a Rust type is an `Option`.
    fn is_option(ty: &syn::Type) -> bool {
        matches!(ty, syn::Type::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "Option"))
    }

    /// The uniffi records whose crate constructor takes their fields in order,
    /// so an array literal fills them by position.
    fn positional_records(model: &Reflection) -> Records {
        model
            .uniffi_records(Build::Android)
            .into_iter()
            .filter(|r| r.positional)
            .map(|r| {
                let fields = r
                    .fields
                    .iter()
                    .map(|(name, ty)| Field {
                        name: name.to_lower_camel_case(),
                        ty: uniffi_type(ty),
                        optional: is_option(ty),
                    })
                    .collect();
                (r.ident, fields)
            })
            .collect()
    }

    /// How many arguments a tuple of n items spreads into, by needle and then n.
    /// A uniffi export takes its trailing `Option` parameters as `null`,
    /// and an extension function falls back to its defaults.
    /// A count where overloads disagree is left out.
    fn plan_spreads(callables: &[Callable]) -> BTreeMap<String, BTreeMap<usize, usize>> {
        let mut widths: BTreeMap<(&str, usize), BTreeSet<usize>> = BTreeMap::new();
        for c in callables {
            let len = c.params.len();
            for n in len - c.defaults - c.nullable..=len {
                let width = if c.nullable > 0 { len } else { n };
                widths.entry((&c.needle, n)).or_default().insert(width);
            }
        }
        let mut spreads: BTreeMap<String, BTreeMap<usize, usize>> = BTreeMap::new();
        for ((needle, n), width) in widths {
            if let [only] = Vec::from_iter(width).as_slice() {
                spreads.entry(needle.to_string()).or_default().insert(n, *only);
            }
        }
        spreads
    }

    /// Finalize the Kotlin output emitted by `convert(_, Lang::Kotlin, _)`.
    /// Takes the JS-shaped intermediate and applies the bulk-text swaps
    /// documented above to land on idiomatic Kotlin.
    pub fn finalize(ir: &str) -> String {
        let js = crate::swift::swap_backticks_for_triple_quotes(ir);
        // Join multi-line expressions (open-paren continuation) into single lines
        // so that per-line rewrites can see the full call.
        let js = join_continuation_lines(&js);

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;
        // Track declared val names to rename duplicates (Kotlin doesn't allow shadowing)
        let mut declared_vals: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        let mut in_string = false;
        for raw in js.lines() {
            let starts_in_string = in_string;
            let quotes = raw.matches("\"\"\"").count();
            if quotes % 2 == 1 {
                in_string = !in_string;
            }
            // A line wholly inside a triple-quoted literal is shader source: keep it as written.
            if starts_in_string && quotes == 0 {
                out.push(raw.to_string());
                continue;
            }
            let line = raw.to_string();
            let trimmed = line.trim_start();

            // Drop JS-only import lines (replaced by the package-level import below)
            if trimmed.starts_with("import ") && trimmed.contains("from \"fragmentcolor\"") {
                if !has_fragmentcolor_import {
                    out.push("import org.fragmentcolor.*".to_string());
                    has_fragmentcolor_import = true;
                }
                continue;
            }

            let line = if starts_in_string { line } else { drop_semicolon_before_comment(&line) };
            let rewritten = rewrite_line(&line);
            // Rename duplicate `val <name>` declarations to avoid conflicting declarations.
            let rewritten = rename_duplicate_val(&rewritten, &mut declared_vals);
            out.push(rewritten);
        }

        let kotlin = out.join("\n");
        check_conflicts(&kotlin);
        kotlin
    }

    /// Fail the build where a call passes, at a position whose receivers disagree,
    /// an argument the skipped rewrite would have changed. Kotlin would not compile it
    /// for the receivers that wanted the rewrite, and the rewriter can't tell them apart.
    fn check_conflicts(kotlin: &str) {
        let conflicts = &facts().conflicts;
        if conflicts.is_empty() {
            return;
        }
        let code = code_only(kotlin);
        for (at, _) in code.char_indices() {
            let Some(needle) = needle_at(&code, at) else {
                continue;
            };
            let open = at + needle.len() - 1;
            let Some(close) = closing(&code, open) else {
                continue;
            };
            let spans = arg_spans(&code[open + 1..close], Nesting::Call);
            let here = conflicts.iter().filter(|c| c.needle == needle && c.argc == spans.len());
            for conflict in here {
                let span = &spans[conflict.position];
                let arg = kotlin[open + 1 + span.start..open + 1 + span.end].trim();
                if (conflict.changes)(arg) {
                    panic!(
                        "kotlin: `{needle}` gets `{arg}` as argument {} of {}, where {}. \
                         Kotlin would not compile one of those calls.",
                        conflict.position + 1,
                        conflict.argc,
                        conflict.reason,
                    );
                }
            }
        }
    }

    fn rewrite_line(src: &str) -> String {
        let mut out = src.to_string();

        if let Some(stripped) = out.trim_end().strip_suffix(';') {
            let leading_len = out.len() - out.trim_start().len();
            let leading = &src[..leading_len];
            out = format!("{}{}", leading, stripped.trim_start());
        }

        out = drop_new_keyword(&out);
        out = replace_leading_keyword(&out, "const ", "val ");
        out = drop_await_prefix(&out);
        out = drop_unwrap(&out);
        out = drop_rust_slice_suffix(&out);
        out = rewrite_array_fill(&out);
        out = swap_single_quoted_strings(&out);
        // Convert [a, b, c] → arrayOf(a, b, c) FIRST so all subsequent
        // API-specific rewrites see a uniform `arrayOf(...)` form.
        // Repeat until stable to handle nested bracket arrays like [[a,b],[c,d]].
        loop {
            let next = rewrite_bracket_array_to_arrayof(&out);
            if next == out { break; }
            out = next;
        }
        out = spread_tuple_args(&out);
        // Before the unsigned pass, which suffixes the index it moves into the call.
        out = rewrite_index_accessors(&out);
        out = rewrite_unsigned_args(&out);
        // Before the generic `arrayOf` → `listOf<Float>` fallback, which
        // would turn integer indices into `0.0f`.
        out = rewrite_list_args(&out);
        out = rewrite_flat_enum_variants(&out);
        out = rewrite_float_args(&out);
        out = rename_stand_ins(&out);
        out = rewrite_arrayof_float_to_listof(&out);
        out = drop_underscore_val(&out);
        // `Shader.new(...)` → `Shader(...)`, after the argument rewrites,
        // which find a constructor call by its `T.new(` needle.
        out = drop_dot_new(&out);
        // Kotlin requires `for (x in y) { ... }`; Rust uses `for x in y { ... }`.
        // Add the parens so the binding declaration parses correctly.
        wrap_for_in_parens(&out)
    }

    /// Rewrites the trimmed arguments of one call, given its needle. `None` keeps them.
    type ArgRewrite<'a> = &'a dyn Fn(&str, &[String]) -> Option<Vec<String>>;

    /// Rewrite the arguments of every call on `line`
    /// whose needle the facts know, inner calls first.
    /// `rewrite` gets the needle and the trimmed arguments,
    /// and returns replacements, possibly a different number.
    fn map_calls(line: &str, rewrite: ArgRewrite) -> String {
        let mut out = String::with_capacity(line.len());
        let mut at = 0;
        while let Some(c) = line[at..].chars().next() {
            let call = needle_at(line, at).and_then(|needle| {
                let open = at + needle.len() - 1;
                closing(line, open).map(|close| (needle, open, close))
            });
            let Some((needle, open, close)) = call else {
                out.push(c);
                at += c.len_utf8();
                continue;
            };
            out.push_str(&line[at..=open]);
            out.push_str(&map_args(&line[open + 1..close], needle, rewrite));
            out.push(')');
            at = close + 1;
        }
        out
    }

    /// The known needle of a call starting at byte `at`.
    fn needle_at(line: &str, at: usize) -> Option<&'static str> {
        let dot = line[at..].starts_with('.');
        if !dot && line[..at].ends_with(|c: char| is_ident_char(c) || c == '.') {
            return None;
        }
        let (ident, end) = read_ident(line, if dot { at + 1 } else { at });
        let after = &line[end..];
        let needle = match (dot, after.starts_with('('), after.starts_with(".new(")) {
            _ if ident.is_empty() => return None,
            (true, true, _) => format!(".{ident}("),
            (false, true, _) => format!("{ident}("),
            (false, false, true) => format!("{ident}.new("),
            _ => return None,
        };
        facts().calls.get_key_value(&needle).map(|(key, _)| key.as_str())
    }

    /// The arguments of one call, rewritten.
    /// Separators and the whitespace around each argument stay as written.
    fn map_args(inner: &str, needle: &str, rewrite: ArgRewrite) -> String {
        let spans = arg_spans(inner, Nesting::Call);
        let args: Vec<String> = spans.iter().map(|s| map_calls(&inner[s.clone()], rewrite)).collect();
        let trimmed: Vec<String> = args.iter().map(|a| a.trim().to_string()).collect();
        let replaced = rewrite(needle, &trimmed);
        if let Some(new) = replaced.as_ref().filter(|new| new.len() != spans.len()) {
            return new.join(", ");
        }
        let mut out = String::with_capacity(inner.len());
        let mut last = 0;
        for (k, span) in spans.iter().enumerate() {
            out.push_str(&inner[last..span.start]);
            let arg = &args[k];
            match &replaced {
                Some(new) => {
                    let lead = arg.len() - arg.trim_start().len();
                    let tail = lead + arg[lead..].trim_end().len();
                    out.push_str(&arg[..lead]);
                    out.push_str(&new[k]);
                    out.push_str(&arg[tail..]);
                }
                None => out.push_str(arg),
            }
            last = span.end;
        }
        out.push_str(&inner[last..]);
        out
    }

    /// The slots of a call to `needle` with `argc` arguments.
    fn slots_of(needle: &str, argc: usize) -> Option<&'static [Slot]> {
        facts().calls.get(needle)?.get(&argc).map(Vec::as_slice)
    }

    /// `arrayOf(...)` → `listOf(...)` where the parameter is a `List`,
    /// with the suffix or constructor its elements need.
    ///
    /// Where the parameter is a type built from a float list,
    /// such as `Vertex`, `arrayOf(numbers)` becomes `Vertex(listOf(...))`.
    /// Where it is a record filled by position, such as `Size`,
    /// `arrayOf(numbers)` becomes `Size(width=8u, height=8u, depth=null)`.
    /// Where every receiver takes a `FloatArray` instead, it becomes `floatArrayOf(...)`.
    fn rewrite_list_args(line: &str) -> String {
        map_calls(line, &|needle, args| {
            let slots = slots_of(needle, args.len())?;
            let new: Vec<String> = args
                .iter()
                .zip(slots)
                .map(|(arg, slot)| {
                    let rewritten = match (&slot.list, &slot.wrap, &slot.record) {
                        (Some(elems), _, _) => array_to_list(&tuple_as_array(arg), elems),
                        (None, Some(ty), _) => scalar_items(arg).map(|items| float_list_of(ty, &items)),
                        (None, None, Some(record)) => scalar_items(arg).and_then(|items| record_of(record, &items)),
                        (None, None, None) if slot.float_array => {
                            scalar_items(arg).map(|items| format!("floatArrayOf({})", float_args(&items)))
                        }
                        (None, None, None) => None,
                    };
                    rewritten.unwrap_or_else(|| arg.clone())
                })
                .collect();
            (new != args).then_some(new)
        })
    }

    /// `(a, b)` → `arrayOf(a, b)`, where a Rust tuple such as the origin of
    /// `ScreenRegion::new((0, 0), (32, 32))` stands for a list. Anything else passes through.
    fn tuple_as_array(arg: &str) -> String {
        let is_tuple = arg.starts_with('(')
            && closing(arg, 0) == Some(arg.len() - 1)
            && split_args(&arg[1..arg.len() - 1]).len() > 1;
        if is_tuple { format!("arrayOf{arg}") } else { arg.to_string() }
    }

    /// `f((a, b))` → `f(a, b, null)`.
    /// A Rust tuple argument spreads over the parameters of the function it calls,
    /// and the trailing `Option` parameters of a uniffi export it leaves out take `null`.
    fn spread_tuple_args(line: &str) -> String {
        map_calls(line, &|needle, args| {
            let [only] = args else {
                return None;
            };
            let inner = only.strip_prefix('(')?.strip_suffix(')')?;
            if closing(only, 0) != Some(only.len() - 1) {
                return None;
            }
            let mut items = split_args(inner);
            if items.len() < 2 {
                return None;
            }
            let width = *facts().spreads.get(needle)?.get(&items.len())?;
            items.resize(width, "null".to_string());
            Some(items)
        })
    }

    /// `Size(width=8u, height=8u, depth=null)` from the items of an `arrayOf(...)`,
    /// or `None` when they can't fill the record's fields in order.
    fn record_of(record: &str, items: &[String]) -> Option<String> {
        let fields = facts().records.get(record)?;
        if items.len() > fields.len() || fields[items.len()..].iter().any(|f| !f.optional) {
            return None;
        }
        let values: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(k, field)| {
                let value = match (items.get(k), &field.ty) {
                    (None, _) => "null".to_string(),
                    (Some(item), KType::Unsigned(ty)) => to_unsigned(item, ty),
                    (Some(item), KType::Float) => floatify_if_numeric(item),
                    (Some(item), _) => item.clone(),
                };
                format!("{}={value}", field.name)
            })
            .collect();
        Some(format!("{record}({})", values.join(", ")))
    }

    /// `arg` as a `listOf(...)`, or `None` when it isn't one `arrayOf(...)`.
    fn array_to_list(arg: &str, elems: &Elems) -> Option<String> {
        let inner = strip_array(arg)?;
        let items: Vec<String> = match elems {
            Elems::AsIs => return Some(format!("listOf({inner})")),
            Elems::Float => row_items(arg).or_else(|| scalar_items(arg))?.iter().map(|i| floatify_if_numeric(i)).collect(),
            Elems::Unsigned => scalar_items(arg)?.iter().map(|i| unsigned_literal(i)).collect(),
            Elems::Rows => split_args(inner)
                .iter()
                .map(|row| scalar_items(row).map_or_else(|| row.clone(), |nums| format!("listOf({})", float_args(&nums))))
                .collect(),
            Elems::Wrap(ty) => split_args(inner)
                .iter()
                .map(|item| scalar_items(item).map_or_else(|| item.clone(), |nums| float_list_of(ty, &nums)))
                .collect(),
        };
        Some(format!("listOf({})", items.join(", ")))
    }

    /// `T(listOf(...))` with every number a `Float`.
    fn float_list_of(ty: &str, items: &[String]) -> String {
        format!("{ty}(listOf({}))", float_args(items))
    }

    /// `items` joined as arguments, with every number a `Float`.
    fn float_args(items: &[String]) -> String {
        let floats: Vec<String> = items.iter().map(|i| floatify_if_numeric(i)).collect();
        floats.join(", ")
    }

    /// The text inside `arg` when it is exactly one `arrayOf(...)`.
    fn strip_array(arg: &str) -> Option<&str> {
        let open = "arrayOf".len();
        (arg.starts_with("arrayOf(") && closing(arg, open) == Some(arg.len() - 1))
            .then(|| &arg[open + 1..arg.len() - 1])
    }

    /// The numbers of an `arrayOf(...)` of number rows, row after row,
    /// the order a flat list such as the matrix of `Model.setTransform` takes them in.
    fn row_items(arg: &str) -> Option<Vec<String>> {
        let rows = split_args(strip_array(arg)?);
        let mut items = Vec::new();
        for row in &rows {
            items.extend(scalar_items(row)?);
        }
        (!rows.is_empty()).then_some(items)
    }

    /// The elements of an `arrayOf(...)` that holds no nested lists.
    fn scalar_items(arg: &str) -> Option<Vec<String>> {
        let items = split_args(strip_array(arg)?);
        let nested = items.iter().any(|i| i.starts_with("arrayOf(") || i.starts_with("listOf("));
        (!nested).then_some(items)
    }

    /// Give number literals an `f` where the parameter is a `Float`.
    fn rewrite_float_args(line: &str) -> String {
        map_calls(line, &|needle, args| {
            let slots = slots_of(needle, args.len())?;
            let new: Vec<String> = args
                .iter()
                .zip(slots)
                .map(|(arg, slot)| {
                    let int_literal = !arg.is_empty() && arg.chars().all(|c| c.is_ascii_digit() || c == '_');
                    if slot.float && !(slot.int && int_literal) {
                        floatify_if_numeric(arg)
                    } else {
                        arg.clone()
                    }
                })
                .collect();
            (new != args).then_some(new)
        })
    }

    /// Give integer literals the `u` Kotlin needs where the parameter is unsigned.
    /// A lone `arrayOf(w, h)` spreads into the separate parameters a Rust `Size` becomes on mobile.
    fn rewrite_unsigned_args(line: &str) -> String {
        map_calls(line, &|needle, args| {
            let plans = facts().calls.get(needle)?;
            let spread = match args {
                [only] if !plans.contains_key(&1) => scalar_items(only).and_then(|items| {
                    let slots = plans.get(&items.len())?;
                    slots
                        .iter()
                        .zip(&items)
                        .map(|(slot, item)| Some(to_unsigned(item, slot.unsigned.as_deref()?)))
                        .collect::<Option<Vec<String>>>()
                }),
                _ => None,
            };
            let spread_out = spread.is_some();
            let args = spread.unwrap_or_else(|| args.to_vec());
            let slots = plans.get(&args.len())?;
            let new: Vec<String> = args
                .iter()
                .zip(slots)
                .map(|(arg, slot)| if slot.unsigned.is_some() { unsigned_literal(arg) } else { arg.clone() })
                .collect();
            (spread_out || new != args).then_some(new)
        })
    }

    /// `64` → `64u`. Anything but an integer literal passes through.
    fn unsigned_literal(arg: &str) -> String {
        let is_literal = arg.starts_with(|c: char| c.is_ascii_digit())
            && arg.chars().all(|c| c.is_ascii_digit() || c == '_');
        if is_literal { format!("{arg}u") } else { arg.to_string() }
    }

    /// A value as the unsigned type `ty`.
    /// `64` → `64u`, `w` → `w.toUInt()`, and other literals stay as written.
    fn to_unsigned(item: &str, ty: &str) -> String {
        if item.starts_with(|c: char| c.is_ascii_digit()) {
            unsigned_literal(item)
        } else {
            format!("{item}.to{ty}()")
        }
    }

    /// `chain.levels()[0]` → `chain.level(0)`,
    /// where the binding reads one element of a Rust slice getter by index.
    fn rewrite_index_accessors(line: &str) -> String {
        let getters: BTreeSet<&str> =
            facts().slice_getters.iter().map(|g| g.getter.as_str()).collect();
        let mut out = line.to_string();
        for getter in getters {
            let needle = format!(".{getter}()[");
            let mut done = String::with_capacity(out.len());
            let mut rest = out.as_str();
            while let Some(at) = rest.find(&needle) {
                let after = &rest[at + needle.len()..];
                let tail = after.trim_start_matches(|c: char| c.is_ascii_digit() || c == '_');
                let index = &after[..after.len() - tail.len()];
                if !index.is_empty() && after[index.len()..].starts_with(']') {
                    done.push_str(&rest[..at]);
                    done.push_str(&format!(".{}({index})", index_accessor(getter, index)));
                    rest = &after[index.len() + 1..];
                } else {
                    done.push_str(&rest[..at + needle.len()]);
                    rest = after;
                }
            }
            done.push_str(rest);
            out = done;
        }
        out
    }

    /// The method every type with a slice getter named `getter` reads one element with.
    /// Fails the build when one of those types has none, as Kotlin has no `getter` to index,
    /// or when they read it through methods of different names, as a line can't type its receiver.
    fn index_accessor(getter: &str, index: &str) -> &'static str {
        let getters: Vec<&SliceGetter> =
            facts().slice_getters.iter().filter(|g| g.getter == getter).collect();
        let missing: Vec<&str> =
            getters.iter().filter(|g| g.accessor.is_none()).map(|g| g.owner.as_str()).collect();
        if !missing.is_empty() {
            panic!(
                "kotlin: `.{getter}()[{index}]` indexes a slice getter of {}, \
                 which the Android bindings leave out without one exported method \
                 that takes an unsigned index and returns the element",
                missing.join(", "),
            );
        }
        let accessors: BTreeSet<&str> =
            getters.iter().filter_map(|g| g.accessor.as_deref()).collect();
        match accessors.into_iter().collect::<Vec<_>>().as_slice() {
            [only] => only,
            _ => {
                let methods: Vec<String> = getters
                    .iter()
                    .filter_map(|g| Some(format!("`{}.{}`", g.owner, g.accessor.as_deref()?)))
                    .collect();
                panic!(
                    "kotlin: `.{getter}()[{index}]` can't become one method call, \
                     since the types with a `{getter}` slice getter read an element through {}, \
                     and a line rewrite can't tell which of them the receiver is",
                    methods.join(", "),
                )
            }
        }
    }

    /// `AlphaMode.Mask` → `AlphaMode.MASK`.
    /// uniffi renders a flat enum as an `enum class` whose entries are in SCREAMING_SNAKE_CASE.
    fn rewrite_flat_enum_variants(line: &str) -> String {
        let mut out = line.to_string();
        for (ident, spellings) in &facts().enums {
            let needle = format!("{ident}.");
            let mut done = String::with_capacity(out.len());
            let mut rest = out.as_str();
            while let Some(at) = rest.find(&needle) {
                done.push_str(&rest[..at]);
                let bounded = !done.ends_with(|c: char| is_ident_char(c) || c == '.');
                done.push_str(&needle);
                rest = &rest[at + needle.len()..];
                let (variant, end) = read_ident(rest, 0);
                if bounded && !variant.is_empty() {
                    let entry = spellings
                        .get(variant)
                        .unwrap_or_else(|| panic!("kotlin: `{ident}.{variant}` names no variant of {ident}"));
                    done.push_str(entry);
                    rest = &rest[end..];
                }
            }
            done.push_str(rest);
            out = done;
        }
        out
    }

    /// Kotlin's `for` loop requires parens around the in-pattern, unlike
    /// Rust's `for x in y { ... }`. Rewrite a line whose first non-whitespace
    /// token is `for ` followed by an identifier, ` in `, an expression, and a
    /// trailing `{`. Conservative: only when the line ends with `{` and the
    /// pattern is a single identifier (not a tuple destructure).
    fn wrap_for_in_parens(line: &str) -> String {
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = line.trim_start();
        if !trimmed.starts_with("for ") {
            return line.to_string();
        }
        let body_open = match trimmed.rfind('{') {
            Some(i) if trimmed[i..].trim() == "{" => i,
            _ => return line.to_string(),
        };
        let head = trimmed[..body_open].trim_end();
        // Split into `for IDENT in REST`.
        let after_for = &head[4..]; // strip `for `
        let in_pos = match after_for.find(" in ") {
            Some(p) => p,
            None => return line.to_string(),
        };
        let ident = after_for[..in_pos].trim();
        let rest = after_for[in_pos + 4..].trim();
        // Already parenthesized? Bail.
        if ident.starts_with('(') {
            return line.to_string();
        }
        // Only rewrite simple identifiers (including `_`).
        if !ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return line.to_string();
        }
        format!("{}for ({} in {}) {{", indent, ident, kotlin_range(rest))
    }

    /// A Rust range in Kotlin: `a..b` → `a until b` and `a..=b` → `a..b`.
    /// Anything else stays as written.
    fn kotlin_range(expr: &str) -> String {
        match expr.split_once("..") {
            Some((lo, hi)) if !hi.contains("..") => match hi.strip_prefix('=') {
                Some(hi) => format!("{}..{}", lo.trim(), hi.trim()),
                None => format!("{} until {}", lo.trim(), hi.trim()),
            },
            _ => expr.to_string(),
        }
    }

    // Drop `.unwrap()` — Kotlin uses exceptions, not Result types.
    fn drop_unwrap(line: &str) -> String {
        line.replace(".unwrap()", "")
    }

    // Drop Rust slice syntax `[..]` and `.asSlice()` after identifiers.
    fn drop_rust_slice_suffix(line: &str) -> String {
        let mut out = line.to_string();
        while let Some(pos) = out.find("[..]") {
            out = format!("{}{}", &out[..pos], &out[pos + 4..]);
        }
        // `.asSlice()` is a Rust-specific method — drop it entirely
        out = out.replace(".asSlice()", "");
        out
    }

    /// Returns `s` with the `f` suffix appended if it's a bare numeric literal
    /// (integer or float, possibly negative). Otherwise returns `s` unchanged.
    fn floatify_if_numeric(s: &str) -> String {
        let t = s.trim();
        try_floatify(t)
            .unwrap_or_else(|| {
                // Not a bare number — try simple binary-op shapes like `a / b` or
                // `current * 0.5`. Walk operators outside of parens and floatify
                // each numeric-looking operand. Identifiers / sub-expressions pass
                // through unchanged.
                if let Some(expr) = floatify_binary_op_operands(t) {
                    expr
                } else {
                    s.to_string()
                }
            })
    }

    fn try_floatify(t: &str) -> Option<String> {
        if t.is_empty() {
            return None;
        }
        if t.ends_with('f') || t.ends_with('F') {
            return Some(t.to_string());
        }
        let body = t.strip_prefix('-').unwrap_or(t);
        if body.is_empty() {
            return None;
        }
        let is_numeric = body.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_');
        if !is_numeric {
            return None;
        }
        Some(if body.contains('.') {
            format!("{}f", t)
        } else {
            format!("{}.0f", t)
        })
    }

    /// Walk a flat arithmetic expression and floatify every numeric operand at
    /// depth 0. Operators handled: `+ - * /`. Identifiers, function calls,
    /// parenthesised sub-expressions pass through unchanged. Returns `None` if
    /// the expression doesn't contain any top-level operator (so the bare-
    /// literal path handles it on its own).
    fn floatify_binary_op_operands(s: &str) -> Option<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut depth = 0i32;
        let mut parts: Vec<String> = Vec::new();
        let mut ops: Vec<char> = Vec::new();
        let mut current = String::new();
        let mut prev_was_op = true; // a leading `-` is unary, not separator
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '(' | '[' | '{' => {
                    depth += 1;
                    current.push(c);
                    prev_was_op = false;
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    current.push(c);
                    prev_was_op = false;
                }
                '+' | '-' | '*' | '/' if depth == 0 && !prev_was_op => {
                    // Treat as separator only if followed by something other
                    // than another operator (avoids `--` / `*=` collapse).
                    let next = chars.get(i + 1).copied().unwrap_or(' ');
                    if next.is_whitespace() || next.is_ascii_digit() || next == '-' || next.is_ascii_alphabetic() || next == '_' || next == '(' {
                        parts.push(current.trim().to_string());
                        current = String::new();
                        ops.push(c);
                        prev_was_op = true;
                        continue;
                    }
                    current.push(c);
                    prev_was_op = false;
                }
                _ => {
                    current.push(c);
                    if !c.is_whitespace() {
                        prev_was_op = false;
                    }
                }
            }
        }
        if ops.is_empty() {
            return None;
        }
        parts.push(current.trim().to_string());
        // Floatify each operand if it's a bare number; otherwise pass through.
        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            let mapped = try_floatify(part).unwrap_or_else(|| part.clone());
            result.push_str(&mapped);
            if let Some(&op) = ops.get(i) {
                result.push(' ');
                result.push(op);
                result.push(' ');
            }
        }
        Some(result)
    }

    /// `[a, b, c]` (JS / Swift array literal) → `arrayOf(a, b, c)` for
    /// Kotlin, since Kotlin only allows `[...]` syntax in annotation
    /// arguments. Bracket-balanced and string-aware so WGSL inside
    /// triple-quoted strings is left alone.
    fn rewrite_bracket_array_to_arrayof(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_dq = false;
        let mut in_tq = false;
        while i < chars.len() {
            // Triple-quote toggle
            if i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
            {
                in_tq = !in_tq;
                out.push_str("\"\"\"");
                i += 3;
                continue;
            }
            if !in_tq && chars[i] == '"' {
                in_dq = !in_dq;
                out.push('"');
                i += 1;
                continue;
            }
            // Skip brackets in annotation positions: `@Foo([...])` is fine.
            // Cheap check: if `[` is preceded by `(` it might still be an
            // expression argument, so we always rewrite outside strings.
            if !in_dq && !in_tq && chars[i] == '[' {
                // Walk to matching `]`.
                let mut depth = 0i32;
                let mut close_pos: Option<usize> = None;
                let mut k = i + 1;
                while k < chars.len() {
                    match chars[k] {
                        '[' => depth += 1,
                        ']' => {
                            if depth == 0 {
                                close_pos = Some(k);
                                break;
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let Some(close) = close_pos {
                    let inner: String = chars[i + 1..close].iter().collect();
                    let inner_trim = inner.trim();
                    // Skip empty `[]` (no items to wrap) and indexer
                    // patterns like `arr[0]` (single ident/expr without
                    // commas — those are subscripts, not collection
                    // literals).
                    if !inner_trim.is_empty() && inner_trim.contains(',') {
                        let prev = if i == 0 { ' ' } else { chars[i - 1] };
                        let prev_is_indexer =
                            prev.is_ascii_alphanumeric() || prev == '_' || prev == ')' || prev == ']';
                        if !prev_is_indexer {
                            out.push_str("arrayOf(");
                            out.push_str(inner_trim);
                            out.push(')');
                            i = close + 1;
                            continue;
                        }
                    }
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// `Array(N).fill(expr)` → `ByteArray(N)` (Kotlin byte array for texture pixel data).
    /// Bracket-balanced; converts all pixel-buffer array-fill patterns to ByteArray.
    fn rewrite_array_fill(line: &str) -> String {
        let needle = "Array(";
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let rem: String = chars[i..].iter().collect();
            if rem.starts_with(needle) {
                let arg_start = i + needle.len();
                let mut depth = 0i32;
                let mut arg_close: Option<usize> = None;
                let mut k = arg_start;
                while k < chars.len() {
                    match chars[k] {
                        '(' => depth += 1,
                        ')' => {
                            if depth == 0 {
                                arg_close = Some(k);
                                break;
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let Some(arg_close) = arg_close {
                    let after: String = chars[arg_close + 1..].iter().collect();
                    if let Some(stripped) = after.strip_prefix(".fill(") {
                        let fill_arg_chars: Vec<char> = stripped.chars().collect();
                        let mut depth2 = 0i32;
                        let mut fill_close: Option<usize> = None;
                        let mut m = 0usize;
                        while m < fill_arg_chars.len() {
                            match fill_arg_chars[m] {
                                '(' => depth2 += 1,
                                ')' => {
                                    if depth2 == 0 {
                                        fill_close = Some(m);
                                        break;
                                    }
                                    depth2 -= 1;
                                }
                                _ => {}
                            }
                            m += 1;
                        }
                        if let Some(fc) = fill_close {
                            let count: String = chars[arg_start..arg_close].iter().collect();
                            // Always produce ByteArray — all pixel-buffer fills are byte arrays
                            out.push_str(&format!("ByteArray({})", count.trim()));
                            i = arg_close + 1 + ".fill(".len() + fc + 1;
                            continue;
                        }
                    }
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Single-quoted JS string → Kotlin double-quoted. Skips content
    /// inside existing `"..."` / `"""..."""` strings.
    fn swap_single_quoted_strings(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_dq = false;
        let mut in_tq = false;
        while i < chars.len() {
            if !in_dq
                && i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
            {
                in_tq = !in_tq;
                out.push_str("\"\"\"");
                i += 3;
                continue;
            }
            if !in_tq && !in_dq && chars[i] == '"' {
                in_dq = true;
                out.push('"');
                i += 1;
                continue;
            }
            if in_dq && chars[i] == '"' {
                in_dq = false;
                out.push('"');
                i += 1;
                continue;
            }
            if !in_dq && !in_tq && chars[i] == '\''
                && let Some(end) = chars[i + 1..].iter().position(|c| *c == '\'') {
                    let inner: String = chars[i + 1..i + 1 + end].iter().collect();
                    if !inner.contains('"') && !inner.contains('\n') {
                        out.push('"');
                        out.push_str(&inner);
                        out.push('"');
                        i += 1 + end + 1;
                        continue;
                    }
                }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    fn drop_await_prefix(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i..].starts_with(&['a', 'w', 'a', 'i', 't', ' ']) {
                let left_ok = i == 0 || !is_ident_char(chars[i - 1]);
                let right_ok =
                    i + 6 < chars.len() && (chars[i + 6].is_alphabetic() || chars[i + 6] == '_');
                if left_ok && right_ok {
                    i += 6; // skip "await "
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    fn replace_leading_keyword(line: &str, from: &str, to: &str) -> String {
        let leading_len = line.len() - line.trim_start().len();
        let (indent, rest) = line.split_at(leading_len);
        if let Some(after) = rest.strip_prefix(from) {
            format!("{}{}{}", indent, to, after)
        } else {
            line.to_string()
        }
    }

    /// Remove `new ` used as a constructor prefix, in code only, never in strings or comments.
    fn drop_new_keyword(line: &str) -> String {
        let code = code_only(line);
        let mut out = String::with_capacity(line.len());
        let (mut copied, mut search) = (0, 0);
        while let Some(found) = code[search..].find("new ") {
            let at = search + found;
            let left_ok = code[..at].chars().next_back().is_none_or(|c| !is_ident_char(c));
            let right_ok = code[at + 4..].chars().next().is_some_and(|c| c.is_alphabetic() || c == '_');
            if left_ok && right_ok {
                out.push_str(&line[copied..at]);
                copied = at + 4;
            }
            search = at + 4;
        }
        out.push_str(&line[copied..]);
        out
    }

    /// Strip `.new(` from `<UpperCamelCase>.new(...)` constructor calls so
    /// they become `<UpperCamelCase>(...)`. The Rust-side `Type::new(args)`
    /// pattern translates to JS as `new Type(args)` (handled by
    /// `drop_new_keyword` above), but the same source emitted via Kotlin
    /// lands as `Type.new(args)` (because `::` → `.` happens before the
    /// rewrite that strips `.new`). Kotlin / Swift use bare-type call
    /// syntax for constructors, so strip the `.new` token.
    ///
    /// Conservative match: requires an identifier whose first character is
    /// uppercase immediately before `.new(`. Lower-case `foo.new(` would be
    /// a method call on an instance and is left alone.
    fn drop_dot_new(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            // Look for `.new(` preceded by an uppercase-led identifier.
            if i + 4 < chars.len()
                && chars[i] == '.'
                && chars[i + 1] == 'n'
                && chars[i + 2] == 'e'
                && chars[i + 3] == 'w'
                && chars[i + 4] == '('
                && i > 0
                && is_ident_char(chars[i - 1])
            {
                // Walk back to the start of the identifier and check that
                // its first character is uppercase (UpperCamelCase = type).
                let mut start = i;
                while start > 0 && is_ident_char(chars[start - 1]) {
                    start -= 1;
                }
                if chars[start].is_ascii_uppercase() {
                    // Skip `.new`, the `(` immediately follows in the output.
                    i += 4;
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Rename duplicate `val <name>` declarations by appending `_N`.
    /// Mutates `declared` to track seen names.
    fn rename_duplicate_val(line: &str, declared: &mut std::collections::HashMap<String, u32>) -> String {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("val ") { return line.to_string(); }
        // Extract the val name (before `=` or `:`)
        let rest = &trimmed["val ".len()..];
        // Find end of name (alphanumeric / underscore)
        let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_').unwrap_or(rest.len());
        if name_end == 0 { return line.to_string(); }
        let name = &rest[..name_end];
        // Skip `_` or names that start with `(`
        if name == "_" || name.starts_with('(') { return line.to_string(); }

        let count = declared.entry(name.to_string()).or_insert(0);
        *count += 1;
        if *count == 1 {
            // First occurrence — no rename needed
            return line.to_string();
        }
        // Rename: replace the first occurrence of `val <name>` with `val <name>_<N>`
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let new_name = format!("{}{}", name, count);
        let renamed_rest = &trimmed["val ".len() + name_end..];
        format!("{}val {}{}", indent, new_name, renamed_rest)
    }

    /// Join continuation lines: when a line ends with more open parens than close parens
    /// (outside strings), the next line is a continuation. Join them with a space so that
    /// per-line rewrites can see the complete expression.
    ///
    /// We skip joining inside triple-quoted strings (WGSL shader code etc.).
    fn join_continuation_lines(src: &str) -> String {
        let mut result_lines: Vec<String> = Vec::new();
        let mut pending: Option<String> = None;
        let mut in_tq = false; // inside triple-quoted string

        for raw in src.lines() {
            // Track triple-quote state
            let mut count_tq = 0usize;
            let chars: Vec<char> = raw.chars().collect();
            let mut i = 0usize;
            while i + 2 < chars.len() {
                if chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                    count_tq += 1;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            if count_tq % 2 == 1 {
                in_tq = !in_tq;
            }

            if in_tq {
                // Inside triple-quoted string — don't join
                if let Some(ref mut p) = pending {
                    p.push('\n');
                    p.push_str(raw);
                } else {
                    result_lines.push(raw.to_string());
                }
                continue;
            }

            // Count paren depth of current line (outside strings)
            let depth = paren_depth_of_line(raw);

            if let Some(ref mut p) = pending {
                // Append continuation to pending, stripping trailing // comments first
                let trimmed_continuation = strip_line_comment(raw.trim());
                p.push(' ');
                p.push_str(trimmed_continuation.trim_end());
                // Recalculate depth of the accumulated line
                let acc_depth = paren_depth_of_line(p);
                if acc_depth <= 0 {
                    result_lines.push(pending.take().unwrap());
                }
            } else if depth > 0 {
                // Start of a multi-line expression; strip trailing comment
                let line_no_comment = strip_line_comment(raw).to_string();
                pending = Some(line_no_comment);
            } else {
                result_lines.push(raw.to_string());
            }
        }
        if let Some(p) = pending {
            result_lines.push(p);
        }
        result_lines.join("\n")
    }

    /// Strip a trailing `//` line comment from a Kotlin/JS line (outside strings).
    fn strip_line_comment(line: &str) -> &str {
        let chars: Vec<char> = line.chars().collect();
        let mut in_str = false;
        let mut in_tq = false;
        let mut i = 0usize;
        while i < chars.len() {
            if !in_str && !in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = !in_tq;
                i += 3; continue;
            }
            if !in_tq && chars[i] == '"' { in_str = !in_str; }
            if !in_str && !in_tq && i + 1 < chars.len()
                && chars[i] == '/' && chars[i+1] == '/' {
                // Find the byte offset for position i
                let byte_pos: usize = line.char_indices()
                    .nth(i).map(|(b, _)| b).unwrap_or(line.len());
                return &line[..byte_pos];
            }
            i += 1;
        }
        line
    }

    /// Count open - close parens for a single line (outside string literals).
    fn paren_depth_of_line(line: &str) -> i32 {
        let mut depth = 0i32;
        let mut in_str = false;
        let mut in_tq = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            if !in_str && !in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = true;
                i += 3;
                continue;
            }
            if in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = false;
                i += 3;
                continue;
            }
            if !in_tq && chars[i] == '"' { in_str = !in_str; }
            if !in_str && !in_tq {
                match chars[i] {
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }
        depth
    }

    /// `arrayOf(1.0, 0.0)` → `listOf(1.0f, 0.0f)` and `arrayOf(255, 0)` → `listOf(255, 0)`
    /// where every element is a number literal. A Rust integer literal is never a float,
    /// so a list of them stays `Int`. Hex bytes stay as written.
    fn rewrite_arrayof_float_to_listof(line: &str) -> String {
        let needle = "arrayOf(";
        if !line.contains(needle) { return line.to_string(); }

        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len() + 32);
        let mut i = 0usize;
        let mut in_tq = false;

        while i < chars.len() {
            // Track triple-quote strings
            if i + 2 < chars.len() && chars[i]=='"' && chars[i+1]=='"' && chars[i+2]=='"' {
                in_tq = !in_tq;
                out.push_str("\"\"\""); i += 3; continue;
            }
            if !in_tq {
                let rem: String = chars[i..].iter().collect();
                if rem.starts_with(needle) {
                    let inner_start = i + needle.len();
                    let mut depth = 0i32;
                    let mut inner_end = None;
                    let mut k = inner_start;
                    while k < chars.len() {
                        match chars[k] {
                            '(' => depth += 1,
                            ')' => { if depth == 0 { inner_end = Some(k); break; } depth -= 1; }
                            _ => {}
                        }
                        k += 1;
                    }
                    if let Some(inner_end) = inner_end {
                        let inner: String = chars[inner_start..inner_end].iter().collect();
                        let elements = split_args(&inner);
                        // Check if all elements are numeric float/int literals (not hex, not strings)
                        let all_numeric = !elements.is_empty() && elements.iter().all(|e| {
                            let e = e.trim().trim_end_matches(',').trim();
                            if e.starts_with("0x") || e.starts_with("0X") { return false; }
                            // Must be a numeric literal (possibly negative, possibly with dot)
                            let e = e.strip_prefix('-').unwrap_or(e);
                            e.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_')
                                && !e.is_empty()
                        });
                        if all_numeric {
                            let integers = elements.iter().all(|e| !e.contains('.'));
                            let items = if integers { elements.join(", ") } else { float_args(&elements) };
                            out.push_str("listOf(");
                            out.push_str(&items);
                            out.push(')');
                            i = inner_end + 1;
                            continue;
                        }
                    }
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// A crate type uniffi doesn't export → the mobile type that stands in for it.
    fn rename_stand_ins(line: &str) -> String {
        facts()
            .stand_ins
            .iter()
            .fold(line.to_string(), |out, s| replace_qualified(&out, &s.ty, &s.stand_in))
    }

    // Drop lines that are `val _ = ...` (Kotlin reserves `_`).
    // Called from kotlin::finalize per-line AFTER rewrite_line.
    fn drop_underscore_val(line: &str) -> String {
        let trimmed = line.trim_start();
        if trimmed.starts_with("val _ =") || trimmed == "val _" {
            return String::new();
        }
        line.to_string()
    }
}
