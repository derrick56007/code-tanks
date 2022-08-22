use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::c_render::Render;

pub fn render(query: Query<(Entity, &Render, &Transform)>) {
    for (entity, render, transform) in &query {
        // println!(
        //     "render {:?}, {:?}, {:?}",
        //     entity.id(),
        //     render.render_type,
        //     position
        // );
    }
}
