use crate::scene::Scene;
use instant::{Duration, Instant};
pub struct Animator {
    pub map: crate::loader::SpriteMap,
    pub cell_counts: mint::Vector2<usize>,
    pub duration: Duration,
    pub sprite: crate::EntityId,
    pub current: mint::Point2<usize>,
    pub moment: Instant,
}

impl Animator {
    pub fn update_uv(&mut self, scene: &mut Scene) {
        let uv_range = self.map.at(self.current);
        scene
            .world
            .get::<&mut crate::Sprite>(self.sprite)
            .unwrap()
            .uv = Some(uv_range);
    }

    pub fn switch<S: Into<usize>>(&mut self, state: usize, scene: &mut Scene) {
        self.moment = Instant::now();
        self.current.x = 0;
        self.current.y = state;
        self.update_uv(scene);
    }

    pub fn tick(&mut self, scene: &mut Scene) {
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
