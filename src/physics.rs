use std::f32;
use bevy::prelude::*;
use crate::resources::*;
use crate::fish::*;
use crate::species::*;
use fishing_game::fishingView::*;
use f32::consts::PI;

const REEL: KeyCode = KeyCode::KeyO;

pub fn simulate_fish(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut fish_info: Query<(&Species, &mut Fish, &mut Transform), With<FishHooked>>,
   // mut rod: Query<(&FishingRod, &Transform, &RotationObj), (With<FishingRod>, With<Rotatable>, Without<FishHooked>)>,
) {
    let (fish_species, mut fish, mut fish_transform) = fish_info.single_mut();
    //let (rod_info, rod_transform, rod_rotation) = rod.single();

    let player_position = Vec3::new(FISHINGROOMX - 100., FISHINGROOMY - WIN_H / 2., 901.);
    
    // Calculate drag
    let p = -fish.position.z;
    let sa = fish.width * fish.width;

    let drag_force = -p * fish_species.cd * sa * fish.velocity * fish.velocity; //Force exerted onto the fish by the water

    fish.forces.drag = drag_force;
    
    // Calculate player force
    let reeling = input.pressed(REEL);

    let player_force = if reeling {
        let delta = player_position - fish.position; //calculate force TWORDS the player

        100. * delta.normalize_or_zero()
    } else {
        Vec3::ZERO
    };

    fish.forces.player = player_force;
    
    // Calculate fish force
    let fish_force: Vec3 = -fish.fish_anger() * fish.velocity.normalize_or_zero(); //opposed velocity
    
    // Calculate net force and acceleration 
    let net_force = drag_force + player_force + fish_force; // fish force works against player drag force works against motion of fish
    let acceleration = net_force / fish.weight;
    fish.velocity = fish.velocity + acceleration * time.delta_seconds();
    //println!("{}", acceleration.to_string());

    // Bounds check
    let mut new_pos = fish.position + fish.velocity * time.delta_seconds();
    
    if new_pos.z > 0. {
        new_pos.z = 0.;
    }
    
    //check for collisions to make sure fish stays on screen
    if new_pos.x < FISHINGROOMX - (WIN_W/2.) + (fish.width) / 2.
    || new_pos.x > FISHINGROOMX + (460.) - (fish.width) / 2.
    {
        println!("conflictx");
        new_pos.x = fish_transform.translation.x;
    }
    
    if new_pos.y > FISHINGROOMY + (WIN_H/2.) - (fish.width) / 2.
    || new_pos.y < FISHINGROOMY - (220.) + (fish.width) / 2.
    {
        println!("conflicty");
        new_pos.y = fish_transform.translation.y;
    }

    fish.position = new_pos;

    //let rod_end = Vec2::new(rod_transform.translation.x + rod_info.length / 2. * f32::cos(rod_rotation.rot + PI / 2.), rod_transform.translation.y + rod_info.length / 2. * f32::sin(rod_rotation.rot + PI / 2.));
    //let fishxy = Vec2::new(fish.position.x, fish.position.y);

    // let dist = (fishxy - rod_end).length();

    // if dist < 5.0
    // {
    //     fish.is_caught = true;
    //     println!("caught fish!");
    // }
}
