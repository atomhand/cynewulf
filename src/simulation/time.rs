use bevy::prelude::*;

#[derive(Resource)]
pub struct SimTime {
    raw_date: u32,
}

impl SimTime {
    pub fn new() -> Self {
        Self { raw_date: 0 }
    }
}

impl SimTime {
    pub fn to_daymonthyear(&self) -> (u32, u32, u32) {
        (
            self.raw_date % 30,
            1 + (self.raw_date % 360) / 30,
            self.raw_date / 360,
        )
    }
}

pub fn tick_date_system(mut sim_time: ResMut<SimTime>) {
    sim_time.raw_date += 1;
}
