struct StoredSystem;

struct Scheduler {
    systems: Vec<StoredSystem>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

trait System<Input> {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>);
}

// expands to:
// impl<F: FnMut()> System<()> for F {}
// impl<F: FnMut(T1, ...T(n) ), T1: 'static> System<(T1)> for F {}
// ... etc
macro_rules! impl_system {
    (
        $(
            $($params:ident),+
        )?
    ) => {
        impl<
            F: FnMut(
                $( $($params),+ )?
            )
            $(, $($params: 'static),+ )?
        >
        System<(
            $( $($params,)+ )?
        )> for F {}
    }
}

impl_system!();
impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);
