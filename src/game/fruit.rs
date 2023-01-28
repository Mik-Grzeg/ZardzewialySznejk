use rand::{prelude::Distribution, distributions::Standard, seq::SliceRandom, thread_rng};

use super::point::Point;

enum Drawer {
    Fruit,
    Nothing,
}

impl Distribution<Drawer> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Drawer {
        match rng.gen::<bool>()  {
            true => Drawer::Fruit,
            false => Drawer::Nothing
        }
    }
}

#[derive(Debug)]
pub struct Fruit {
    point: Point
}

impl Fruit {
    pub fn try_spawn_at_random_place(filtered_out_occupied_points: &Vec<Point>) -> Option<Self> {
        match rand::random() {
            Drawer::Fruit => {
                let point = *filtered_out_occupied_points.choose(&mut thread_rng())?;

                Some(Self {
                    point
                })
            },
            Drawer::Nothing => None
        }
    }
}
