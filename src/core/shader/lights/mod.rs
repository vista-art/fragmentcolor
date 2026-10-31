//! The lights a shader with light roots takes, and its ambient term.

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Weak};

use super::ShaderObject;
use crate::PassError;
use crate::scene::{GpuLight, LightObject, bump_epoch, current_epoch};

pub(crate) mod roots;

/// The most lights one shader takes on every device this process opened.
static LIGHT_CAP: AtomicU32 = AtomicU32::new(u32::MAX);

#[cfg(test)]
thread_local! {
    /// A cap for the current test thread alone, so a test that forces a small
    /// device cap leaves the tests running beside it alone.
    static TEST_CAP: std::cell::Cell<u32> = const { std::cell::Cell::new(u32::MAX) };
}

/// Lowers the process-wide light cap to what a new device takes.
pub(crate) fn lower_light_cap(cap: u32) {
    LIGHT_CAP.fetch_min(cap, Ordering::Relaxed);
}

/// The most lights one shader takes on every device this process opened.
pub(crate) fn light_cap() -> u32 {
    let cap = LIGHT_CAP.load(Ordering::Relaxed);
    #[cfg(test)]
    let cap = cap.min(TEST_CAP.with(std::cell::Cell::get));
    cap
}

/// Lowers the light cap for the calling test thread only.
#[cfg(test)]
pub(crate) fn lower_test_cap(cap: u32) {
    TEST_CAP.with(|current| current.set(current.get().min(cap)));
}

/// The fewest entries a light list holds before it drops the dead ones.
const PRUNE_FLOOR: usize = 64;

/// The lights that reach one shader, in attach order, and its ambient term.
#[derive(Debug, Default)]
pub(crate) struct ShaderLights {
    /// Light ids beside weak handles, so a lookup by id upgrades nothing.
    /// Entries whose light dropped stay until the next prune.
    list: Vec<(u64, Weak<LightObject>)>,
    /// The ids in `list`, so adding a light takes constant time.
    ids: HashSet<u64>,
    /// The length of `list` that drops the dead entries next.
    prune_at: usize,
    ambient: [f32; 3],
    /// Whether the shader draws with `GpuLight::placeholder` while no light reaches it.
    placeholder: bool,
    /// The summary at the light epoch it was taken at.
    summary: Option<(u64, LightSummary)>,
}

/// What a render reads of a shader's lights. Every change to them moves
/// the light epoch, so one summary holds for the epoch it was taken at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LightSummary {
    /// A hash of the live light ids in order, the ambient term and the placeholder flag.
    pub(crate) hash: u64,
    /// The number of live lights.
    pub(crate) count: u64,
}

impl ShaderLights {
    /// Drops the entries whose light dropped.
    fn prune(&mut self) {
        self.list.retain(|(_, weak)| weak.strong_count() > 0);
        self.ids = self.list.iter().map(|(id, _)| *id).collect();
        self.prune_at = (2 * self.list.len()).max(PRUNE_FLOOR);
    }

    /// Adds `light` once, and returns whether it was new.
    fn push(&mut self, light: &Arc<LightObject>) -> bool {
        if !self.ids.insert(light.id) {
            return false;
        }
        self.list.push((light.id, Arc::downgrade(light)));
        if self.list.len() >= self.prune_at {
            self.prune();
        }
        true
    }
}

/// The lights a lit shader packs at one render.
#[derive(Debug)]
struct LightList {
    /// The live lights in attach order.
    lights: Vec<Arc<LightObject>>,
    placeholder: bool,
}

impl ShaderObject {
    /// Adds `light` to this shader's list, once. A shader without light roots ignores it.
    pub(crate) fn add_light(&self, light: &Arc<LightObject>) {
        if self.light_roots.is_none() {
            return;
        }
        let mut lights = self.lights.write();
        let added = lights.push(light);
        drop(lights);
        if added {
            bump_epoch();
        }
    }

    /// Runs `attach`, then takes the lights `incoming` lists, under one hold of the shader's
    /// lights. They must fit next to the lights the shader holds, which is checked first,
    /// so a refusal from the check or from `attach` changes no light, and no light reaches
    /// the shader from another pass in between.
    pub(crate) fn join_lights(
        &self,
        incoming: &[&Arc<LightObject>],
        attach: impl FnOnce() -> Result<(), PassError>,
    ) -> Result<(), PassError> {
        let Some(roots) = self.light_roots.as_ref() else {
            return attach();
        };
        let mut lights = self.lights.write();
        let ids: Vec<u64> = incoming.iter().map(|light| light.id).collect();
        check_room(&mut lights, roots.capacity(), &ids)?;
        attach()?;
        let mut added = false;
        for light in incoming {
            added |= lights.push(light);
        }
        drop(lights);
        if added {
            bump_epoch();
        }
        Ok(())
    }
}

/// Hands `light` to every shader of `shaders` with light roots, or to none when one is full.
/// It holds every one of their light lists from the check to the push, taken in id order,
/// so no light lands between the two, and two passes that share shaders can't deadlock.
pub(crate) fn share_light(
    shaders: &[Arc<ShaderObject>],
    light: &Arc<LightObject>,
) -> Result<(), PassError> {
    let mut lit: Vec<&Arc<ShaderObject>> = shaders
        .iter()
        .filter(|shader| shader.light_roots.is_some())
        .collect();
    lit.sort_by_key(|shader| shader.id);
    lit.dedup_by_key(|shader| shader.id);
    let mut held = Vec::with_capacity(lit.len());
    for shader in lit {
        let capacity = shader
            .light_roots
            .as_ref()
            .map_or(u32::MAX, |roots| roots.capacity());
        let mut lights = shader.lights.write();
        check_room(&mut lights, capacity, &[light.id])?;
        held.push(lights);
    }
    #[cfg(test)]
    crate::testing::race_hooks::run("light checked");
    let mut added = false;
    for lights in held.iter_mut() {
        added |= lights.push(light);
    }
    drop(held);
    if added {
        bump_epoch();
    }
    Ok(())
}

/// Checks that the lights `incoming` names, each named once, fit next to the ones in
/// `lights`, a list of a shader whose light roots hold `capacity` lights.
fn check_room(lights: &mut ShaderLights, capacity: u32, incoming: &[u64]) -> Result<(), PassError> {
    let cap = light_cap().min(capacity);
    let fresh = incoming
        .iter()
        .filter(|id| !lights.ids.contains(id))
        .count();
    let over = |lights: &ShaderLights| (lights.list.len() + fresh) as u64 > u64::from(cap);
    // The list counts dropped lights until a prune, so count exactly only when it looks full.
    if over(lights) {
        lights.prune();
        if over(lights) {
            return Err(PassError::LightCapReached { cap });
        }
    }
    Ok(())
}

impl ShaderObject {
    /// The hash and count of the live lights, taken once per light epoch.
    pub(crate) fn light_summary(&self) -> LightSummary {
        // Read the epoch first, so a change that lands meanwhile moves it past the stored one.
        let epoch = current_epoch();
        if let Some((taken, summary)) = self.lights.read().summary
            && taken == epoch
        {
            return summary;
        }
        let mut lights = self.lights.write();
        let mut hasher = DefaultHasher::new();
        let mut count = 0;
        for (id, weak) in &lights.list {
            if weak.strong_count() > 0 {
                id.hash(&mut hasher);
                count += 1;
            }
        }
        lights.ambient.map(f32::to_bits).hash(&mut hasher);
        lights.placeholder.hash(&mut hasher);
        let summary = LightSummary {
            hash: hasher.finish(),
            count,
        };
        lights.summary = Some((epoch, summary));
        summary
    }

    /// The live lights and the placeholder flag.
    fn light_list(&self) -> LightList {
        let lights = self.lights.read();
        LightList {
            lights: lights
                .list
                .iter()
                .filter_map(|(_, weak)| weak.upgrade())
                .collect(),
            placeholder: lights.placeholder,
        }
    }

    /// Sets the ambient term the shader reads as `lights.ambient`.
    pub(crate) fn set_ambient(&self, color: [f32; 3]) {
        self.lights.write().ambient = color;
        bump_epoch();
    }

    /// The ambient term the shader reads as `lights.ambient`.
    pub(crate) fn ambient(&self) -> [f32; 3] {
        self.lights.read().ambient
    }

    /// Makes the shader draw with the placeholder light while no light reaches it.
    pub(crate) fn set_placeholder(&self) {
        self.lights.write().placeholder = true;
        bump_epoch();
    }

    /// The lights this shader draws with, in attach order: its live lights,
    /// or the placeholder when it has one and no light reaches it.
    pub(crate) fn gpu_lights(&self) -> Vec<GpuLight> {
        let list = self.light_list();
        if list.lights.is_empty() && list.placeholder {
            return vec![GpuLight::placeholder()];
        }
        list.lights.iter().map(|light| light.gpu()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::scene::{Light, bump_epoch, current_epoch};

    // Story: a shader's light summary holds for the light epoch it was taken at,
    // so a render reads it without walking the lights, and the next epoch takes it again.
    #[test]
    fn the_light_summary_holds_for_one_epoch() {
        let shader = crate::Material::pbr().shader().object.clone();
        let lamp = Light::point([0.0, 0.0, 0.0], [1.0; 3]);
        let push = || {
            let entry = (lamp.object.id, Arc::downgrade(&lamp.object));
            shader.lights.write().list.push(entry);
        };
        // A list change that skips the epoch stays out of the summary. Another
        // test can move the epoch between the reads, so a few tries are allowed.
        let held = (0..64).any(|_| {
            let epoch = current_epoch();
            let before = shader.light_summary();
            push();
            let during = shader.light_summary();
            shader.lights.write().list.pop();
            current_epoch() == epoch && during == before
        });
        assert!(held, "the summary walked the lights again within one epoch");

        push();
        bump_epoch();
        assert_eq!(
            shader.light_summary().count,
            1,
            "a new epoch counts the lamp"
        );
    }
}
