//! The lights a shader with light roots takes, and its ambient term.

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Weak};

use super::ShaderObject;
use crate::PassError;
use crate::scene::{GpuLight, LightObject, bump_epoch};

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

/// The lights that reach one shader, in attach order, and its ambient term.
#[derive(Debug, Default)]
pub(crate) struct ShaderLights {
    /// Light ids beside weak handles, so a lookup by id upgrades nothing.
    list: Vec<(u64, Weak<LightObject>)>,
    ambient: [f32; 3],
    /// Whether the shader draws with `GpuLight::placeholder` while no light reaches it.
    placeholder: bool,
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
        lights.list.retain(|(_, weak)| weak.strong_count() > 0);
        if lights.list.iter().any(|(id, _)| *id == light.id) {
            return;
        }
        lights.list.push((light.id, Arc::downgrade(light)));
        drop(lights);
        bump_epoch();
    }

    /// Checks that the lights `incoming` names fit next to the ones this shader holds.
    pub(crate) fn check_light_room(&self, incoming: &[u64]) -> Result<(), PassError> {
        let Some(roots) = self.light_roots.as_ref() else {
            return Ok(());
        };
        let cap = light_cap().min(roots.capacity());
        let lights = self.lights.read();
        let mut held: Vec<u64> = lights
            .list
            .iter()
            .filter(|(_, weak)| weak.strong_count() > 0)
            .map(|(id, _)| *id)
            .collect();
        for id in incoming {
            if !held.contains(id) {
                held.push(*id);
            }
        }
        if held.len() as u64 > u64::from(cap) {
            return Err(PassError::LightCapReached { cap });
        }
        Ok(())
    }

    /// The ids of the live lights, in attach order.
    pub(crate) fn light_ids(&self) -> Vec<u64> {
        self.lights
            .read()
            .list
            .iter()
            .filter(|(_, weak)| weak.strong_count() > 0)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Feeds the live light ids, the ambient term and the placeholder flag to `hasher`.
    pub(crate) fn hash_lights(&self, hasher: &mut impl Hasher) {
        let lights = self.lights.read();
        for (id, weak) in &lights.list {
            if weak.strong_count() > 0 {
                id.hash(hasher);
            }
        }
        lights.ambient.map(f32::to_bits).hash(hasher);
        lights.placeholder.hash(hasher);
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
