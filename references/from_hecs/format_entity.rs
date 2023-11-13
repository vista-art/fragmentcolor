type FormattingFunction = &'static dyn Fn(hecs::ObjectId<'_>) -> Option<String>;

fn format_entity(entity: hecs::ObjectId<'_>) -> String {
    fn fmt<T: hecs::Component + std::fmt::Display>(entity: hecs::ObjectId<'_>) -> Option<String> {
        Some(entity.get::<&T>()?.to_string())
    }

    const FUNCTIONS: &[FormattingFunction] = &[&fmt::<i32>, &fmt::<bool>, &fmt::<f64>];

    let mut out = String::new();
    for f in FUNCTIONS {
        if let Some(x) = f(entity) {
            if out.is_empty() {
                out.push('[');
            } else {
                out.push_str(", ");
            }
            out.push_str(&x);
        }
    }
    if out.is_empty() {
        out.push_str("[]");
    } else {
        out.push(']');
    }
    out
}

//usage
//
// fn main() {
//     let mut world = hecs::World::new();
//     let e = world.spawn((42, true));
//     println!("{}", format_entity(world.entity(e).unwrap()));
// }
