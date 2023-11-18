// @TODO Make this component work with a SceneObject<Sprite>
//       with a cleaner interface. Remove this comment when you see it on screen.

use crate::{
    components::{Sprite, SpriteMap},
    scene::SceneState,
};
use instant::{Duration, Instant};

pub struct Animator {
    pub cell_counts: mint::Vector2<usize>,
    pub current: mint::Point2<usize>,
    pub sprite: crate::ObjectId,
    pub sprite_map: SpriteMap,
    pub duration: Duration,
    pub moment: Instant,
}

impl Animator {
    pub fn update_uv(&mut self, scene: &mut SceneState) {
        let clip_region = self.sprite_map.at(self.current);
        scene.get::<&mut Sprite>(self.sprite).unwrap().clip_region = Some(clip_region);
    }

    pub fn switch<S: Into<usize>>(&mut self, state: usize, scene: &mut SceneState) {
        self.moment = Instant::now();
        self.current.x = 0;
        self.current.y = state;
        self.update_uv(scene);
    }

    pub fn tick(&mut self, scene: &mut SceneState) {
        if self.moment.elapsed() < self.duration {
            return;
        }

        self.current.x += 1;
        self.moment = Instant::now();
        if self.current.x == self.cell_counts.x {
            self.current.x = 0;
            self.current.y = 0;
            // don't update the scene here, so that
            // input can have a chance to transition
            // to something other than 0 (Idle).
        } else {
            self.update_uv(scene);
        }
    }
}
