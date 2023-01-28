use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};

use super::point::Point;

#[derive(Debug, PartialEq)]
pub struct Fruit {
    pub point: Point,
}

impl Fruit {
    pub fn try_spawn_at_random_place(
        filtered_out_occupied_points: &Vec<&Point>,
        current_number_of_fruits: usize,
    ) -> Option<Self> {
        // Lower the chance of spawning new fruit if there is already plenty of them on the board
        if thread_rng().gen_range(0.0..=1.0) < 0.08 / (current_number_of_fruits + 1) as f32 {
            let point = **filtered_out_occupied_points.choose(&mut thread_rng())?;

            Some(Self { point })
        } else {
            None
        }
    }
}
