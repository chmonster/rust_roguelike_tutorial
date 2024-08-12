use rltk::RGB;
use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

//#[derive(Component)]
//struct RandomMover {}

#[derive(Component, Debug)]
struct Player {}
