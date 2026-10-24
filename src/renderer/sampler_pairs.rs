use crate::TextureObject;
use crate::texture::{CompareFunction, SamplerOptions, create_sampler};
use std::sync::Arc;

/// The sampled textures and the sampler bindings of one bind group.
#[derive(Default)]
pub(super) struct SamplerSources {
    textures: Vec<(u32, Arc<TextureObject>)>,
    samplers: Vec<(u32, bool)>,
}

impl SamplerSources {
    pub(super) fn add_texture(&mut self, binding: u32, texture: Arc<TextureObject>) {
        self.textures.push((binding, texture));
    }

    pub(super) fn add_sampler(&mut self, binding: u32, comparison: bool) {
        self.samplers.push((binding, comparison));
    }

    /// Builds each sampler binding from the options of the texture paired with it.
    /// A sampler with no texture in its group uses the default options.
    pub(super) fn resolve(mut self, device: &wgpu::Device) -> Vec<(u32, wgpu::Sampler)> {
        self.textures.sort_by_key(|(binding, _)| *binding);
        self.samplers.sort_by_key(|(binding, _)| *binding);
        let bindings: Vec<u32> = self.textures.iter().map(|(binding, _)| *binding).collect();

        let mut previous = None;
        let mut samplers = Vec::with_capacity(self.samplers.len());
        for &(binding, comparison) in &self.samplers {
            let paired = paired_texture(binding, previous, &bindings);
            let sampler = match paired.and_then(|index| self.textures.get(index)) {
                Some((_, texture)) => texture.sampler_for(device, comparison),
                None => {
                    let compare = comparison.then_some(CompareFunction::LessEqual);
                    create_sampler(
                        device,
                        SamplerOptions {
                            compare,
                            ..Default::default()
                        },
                    )
                }
            };
            samplers.push((binding, sampler));
            previous = Some(binding);
        }
        samplers
    }
}

/// Index of the texture the sampler at `binding` takes its options from.
/// That is the first texture after the `previous` sampler binding,
/// or the nearest texture in the group when no texture sits between the two.
fn paired_texture(binding: u32, previous: Option<u32>, textures: &[u32]) -> Option<usize> {
    let after_previous = |texture: u32| previous.is_none_or(|previous| texture > previous);
    let between = textures
        .iter()
        .position(|&texture| texture < binding && after_previous(texture));
    between.or_else(|| {
        let nearest = textures.iter().enumerate();
        nearest
            .min_by_key(|(_, texture)| texture.abs_diff(binding))
            .map(|(index, _)| index)
    })
}

#[cfg(test)]
mod tests {
    use super::paired_texture;

    // Story: Each sampler in an interleaved group pairs with the texture just before it.
    #[test]
    fn interleaved_samplers_pair_with_their_own_texture() {
        let textures = [0, 2];
        assert_eq!(paired_texture(1, None, &textures), Some(0));
        assert_eq!(paired_texture(3, Some(1), &textures), Some(1));
    }

    // Story: One sampler after several textures takes the first of them, like the PBR group.
    #[test]
    fn shared_sampler_pairs_with_the_first_texture() {
        assert_eq!(paired_texture(5, None, &[0, 1, 2, 3, 4]), Some(0));
    }

    // Story: A sampler with no texture before it takes the nearest one.
    #[test]
    fn sampler_without_a_texture_before_it_takes_the_nearest() {
        assert_eq!(paired_texture(0, None, &[1, 4]), Some(0));
        assert_eq!(paired_texture(3, Some(2), &[0, 1]), Some(1));
        assert_eq!(paired_texture(0, None, &[]), None);
    }
}
