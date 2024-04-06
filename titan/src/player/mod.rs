mod systems;

use bevy::prelude::*;

use crate::AppState;

use self::systems::{draw_cursor, setup};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, draw_cursor.run_if(in_state(AppState::InGame)));
    }
}
