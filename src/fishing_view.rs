extern crate rand;

use crate::fish::*;
use crate::gameday::*;
use crate::interface::*;
use crate::inventory::*;
use crate::map::*;
use crate::physics::*;
use crate::prob_calc::*;
use crate::resources::*;
use crate::species::*;
use crate::weather::*;
use crate::window::*;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::render::view::visibility;
use bevy::sprite::*;
use bevy::sprite::*;
use bevy::sprite::*;
use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;
use std::f32;
use std::f32::consts::PI;
use std::hash::{Hash, Hasher};
use std::time::Duration;

const TUG: KeyCode = KeyCode::KeyP;
const REEL: KeyCode = KeyCode::KeyO;
const ROTATE_ROD_COUNTERLCOCKWISE: KeyCode = KeyCode::KeyA;
const ROTATE_ROD_CLOCKWISE: KeyCode = KeyCode::KeyD;
const SWITCH_ROD: KeyCode = KeyCode::KeyN;
const SWITCH_LINE: KeyCode = KeyCode::KeyM;
const SWITCH_LURE: KeyCode = KeyCode::KeyX;

pub const PARTICLECOUNT: usize = 10;

const CATCH_MARGIN: f32 = 30.;

const DEPTH_DECAY: f32 = 40.;

pub const FISHING_ROOM_CENTER: Vec2 = Map::get_area_center(0, -2);
pub const FISHING_ROOM_X: f32 = FISHING_ROOM_CENTER.x;
pub const FISHING_ROOM_Y: f32 = FISHING_ROOM_CENTER.y;

pub const PLAYER_POSITION: Vec3 = Vec3::new(
    FISHING_ROOM_X - 100.,
    FISHING_ROOM_Y - (WIN_H / 2.) + 50.,
    902.,
);

const POWER_BAR_Y_OFFSET: f32 = FISHING_ROOM_Y - 308.;
const MAX_POWER: f32 = 250.;
const POWER_FILL_SPEED: f32 = 250.;

const ROD_MIN_ROTATION: f32 = PI / 6.;
const ROD_MAX_ROTATION: f32 = 5. / 6. * PI;
const ROD_ROTATION_SPEED: f32 = PI / 2.;

const MAX_CAST_DISTANCE: f32 = 400.;
const CASTING_SPEED: f32 = 250.;
const REEL_IN_SPEED: f32 = 150.;

lazy_static! {
    static ref RODS: HashMap<&'static str, &'static FishingRodType> = {
        let mut map = HashMap::new();
        map.insert("Default Rod", &FishingRodType::NORMAL);
        map.insert("Surf Rod", &FishingRodType::SURF);
        map
    };
    static ref LINES: HashMap<&'static str, &'static FishingLineType> = {
        let mut map = HashMap::new();
        map.insert("FluoroCarbon Fishing Line", &FishingLineType::FLUOROCARBON);
        map.insert("Braided Fishing Line", &FishingLineType::BRAIDED);
        map.insert("Monofilament Fishing Line", &FishingLineType::MONOFILILMENT);
        map.insert("Golden Fishing Line", &FishingLineType::GOLDEN);
        map
    };
    static ref LURES: HashMap<&'static str, &'static Lure> = {
        let mut map = HashMap::new();
        map.insert("Bobber", &Lure::BOBBER);
        map.insert("Frog Bait", &Lure::FROG);
        map.insert("Swim Bait", &Lure::FISH);
        map
    };
}

#[derive(Resource)]
pub struct StartFishingAnimation {
    pub active: bool,
    pub button_control_active: bool,
}

#[derive(Resource)]
pub struct FishingAnimationDuration(pub Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FishingState {
    #[default]
    Idle,
    Casting,
    ReelingUnhooked,
    ReelingHooked,
}

#[derive(Component, States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FishingLocal {
    #[default]
    Pond1,
    Pond2,
    Ocean,
}
#[derive(Component)]
pub struct HookedDebris {
    pub drag_increase: f32,
}

#[derive(Component)]
pub struct DebrisHooked {
    pub hooked: bool,
}

#[derive(Component)]
struct LureHUD;

#[derive(Component)]
struct PondScreen;

#[derive(Component)]
struct BeachScreen;

#[derive(Component)]
struct PowerBar {
    power: f32,
}

#[derive(Component)]
pub struct MysteryFish;

#[derive(Component)]
struct PhysicsFish;

#[derive(Component)]
pub struct FishingRod {
    pub rod_type: &'static FishingRodType,
    pub rotation: f32,
    pub material: Handle<ColorMaterial>,
    pub segments: Vec<Entity>,
    pub tip_pos: Vec3,
}

#[derive(Component, Default)]
pub struct ParticleList {
    pub particle_list: Vec<Particle>,
}

#[derive(Component, Default)]
struct Bobber;

#[derive(Component, Clone)]

pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
}

impl Particle {
    pub const fn new(position: Vec3, velocity: Vec3, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }
}

impl Hash for Particle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.x as i32;
        self.position.y as i32;
        self.position.z as i32;
    }
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.position.x as i32 == other.position.x as i32
            && self.position.y as i32 == other.position.y as i32
            && self.position.z as i32 == other.position.z as i32
    }
}

impl Eq for Particle {}

#[derive(Component)]
struct FishingRodSegment;

pub struct FishingRodType {
    pub texture: &'static str,
    pub length: f32,
    pub radius: f32,
    pub thickness: f32,
    pub flexural_strength: f32,
    pub shear_modulus: f32,
    pub blank_color: Color,
}

impl FishingRodType {
    pub const fn new(
        texture: &'static str,
        length: f32,
        radius: f32,
        thickness: f32,
        shear_strength: f32,
        shear_modulus: f32,
        blank_color: Color,
    ) -> Self {
        Self {
            texture,
            length,
            radius,
            thickness,
            flexural_strength: shear_strength,
            shear_modulus,
            blank_color,
        }
    }

    pub const NORMAL: FishingRodType = FishingRodType::new(
        "rods/default.png",
        0.75,
        0.015,
        0.004,
        3450E6,
        72E9,
        Color::BLACK,
    );
    pub const SURF: FishingRodType = FishingRodType::new(
        "rods/surf.png",
        1.,
        0.015,
        0.004,
        3450E6,
        72E9,
        Color::BLACK,
    );
}

#[derive(Resource)]
struct DirectionTimer {
    pub timer: Timer,
}

#[derive(Resource)]
struct ExclamationTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct FishingLine {
    pub cast_distance: f32,
    pub length: f32,
    pub start: Vec3,
    pub end: Vec3,
    pub segments: Vec<Entity>,
    pub line_type: &'static FishingLineType,
}

impl FishingLine {
    pub fn new(line_type: &'static FishingLineType) -> Self {
        Self {
            cast_distance: 0.0,
            length: 0.0,
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            segments: Vec::new(),
            line_type,
        }
    }

    pub const WIDTH: f32 = 1.;
}

#[derive(PartialEq)]
pub struct FishingLineType {
    pub ultimate_tensile_strength: f32,
    pub color: Color,
}

impl FishingLineType {
    pub const fn new(ultimate_tensile_strength: f32, color: Color) -> Self {
        Self {
            ultimate_tensile_strength,
            color,
        }
    }

    pub const FLUOROCARBON: FishingLineType =
        FishingLineType::new(3000., Color::srgb(0.1, 0.1, 0.8));
    pub const BRAIDED: FishingLineType = FishingLineType::new(4000., Color::srgb(0.0, 0.7, 0.2));
    pub const MONOFILILMENT: FishingLineType =
        FishingLineType::new(2000., Color::srgb(0.9, 0.9, 0.9));
    pub const GOLDEN: FishingLineType = FishingLineType::new(10000., Color::srgb(0.88, 0.77, 0.25));
}

#[derive(Component)]
struct FishingLineSegment;

#[derive(Component, Default, Clone, Copy)]
pub struct Lure {
    pub texture_index: usize,
    pub mass: f32,
    pub depth: f32,
    pub cd: (f32, f32),
    pub sa: (f32, f32),
    pub name: &'static str,
}

impl Lure {
    pub const fn new(
        texture_index: usize,
        mass: f32,
        depth: f32,
        cd: (f32, f32),
        sa: (f32, f32),
        name: &'static str,
    ) -> Self {
        Self {
            texture_index,
            mass,
            depth,
            cd,
            sa,
            name,
        }
    }

    pub const BOBBER: Lure = Lure::new(0, 2.0, 1., (0.47, 0.47), (50., 50.), "Bobber");
    pub const FROG: Lure = Lure::new(1, 2.0, 20., (0.14, 1.14), (40., 90.), "Frog Bait");
    pub const FISH: Lure = Lure::new(2, 2.0, 150., (0.09, 0.86), (35., 70.), "Swim Bait");
}

#[derive(Component)]
struct Wave;

#[derive(Component, Default)]
struct Splash {
    pub position: Vec3,
}

#[derive(Component)]
pub struct InPond;

#[derive(Component)]
pub struct IsBass;

#[derive(Component)]
pub struct exclam_point;

#[derive(Component)]
pub struct PondObstruction;

#[derive(Component, PartialEq)]
pub enum ObstType {
    Tree,
    Fissure,
    Pad,
    Debris,
}

#[derive(Component)]
pub struct DebrisType {
    pub mass: f32,
    pub drag_increase: f32,
    pub width: f32,
    pub height: f32,
}

impl DebrisType {
    pub const fn new(mass: f32, drag_increase: f32, width: f32, height: f32) -> Self {
        Self {
            mass,
            drag_increase,
            width,
            height,
        }
    }

    pub const WATER_BOTTLE: DebrisType = DebrisType::new(0.4, 0.3, 30., 42.);
    pub const BUSH: DebrisType = DebrisType::new(2.0, 0.5, 32., 40.);
}

//FISH THING
#[derive(Component)]
struct FishDetails {
    pub name: &'static str,
    pub fish_id: i32,
    pub length: i32,
    pub width: i32,
    pub weight: i32,
    pub time_of_day: (usize, usize),
    pub weather: Weather,
    //bounds
    pub depth: (i32, i32),
    //x, y, z
    pub position: (i32, i32),
    pub change_x: Vec3,
    pub change_y: Vec3,
    //length, width, depth
    pub bounds: (i32, i32),
    pub hunger: f32,
    pub touching_lure: bool,
}

impl FishDetails {
    pub fn new(
        name: &'static str,
        fish_id: i32,
        length: i32,
        width: i32,
        weight: i32,
        time_of_day: (usize, usize),
        weather: Weather,
        depth: (i32, i32),
        position: (i32, i32),
        change_x: Vec3,
        change_y: Vec3,
        bounds: (i32, i32),
        hunger: f32,
        touching_lure: bool,
    ) -> Self {
        Self {
            name,
            fish_id,
            length,
            width,
            weight,
            time_of_day,
            weather,
            depth,
            position,
            change_x,
            change_y,
            bounds,
            hunger,
            touching_lure,
        }
    }
}

#[derive(Component)]
pub struct FishingViewPlugin;

impl Plugin for FishingViewPlugin {
    fn build(&self, app: &mut App) {

        app.init_state::<FishingState>()
            .insert_resource(ProbTimer::new(2.))
            .insert_resource(FishVisibiltyUpdated(false))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    move_fish,
                    update_fish_visibility.run_if(|visibilty_updated: Res<FishVisibiltyUpdated>| !visibilty_updated.0),
                    fish_area_lure
                        .run_if(in_state(FishingState::ReelingUnhooked))
                        .after(move_fish),
                    (power_bar_cast, switch_rod, switch_line, switch_lure)
                        .run_if(in_state(FishingState::Idle)),
                    rod_rotate.after(fish_area_lure),
                    (
                        calculate_water_force,
                        (calculate_buoyancy_force, calculate_player_force).run_if(
                            in_state(FishingState::ReelingUnhooked)
                                .or_else(in_state(FishingState::ReelingHooked)),
                        ),
                    )
                        .after(rod_rotate),
                    calculate_fish_force
                        .after(calculate_water_force)
                        .after(calculate_buoyancy_force)
                        .after(calculate_player_force),
                    simulate_physics.after(calculate_fish_force),
                    (
                        bend_fishing_rod,
                        handle_debris.run_if(
                            in_state(FishingState::ReelingUnhooked)
                                .or_else(in_state(FishingState::ReelingHooked)),
                        ),
                    )
                        .after(simulate_physics),
                    (
                        is_done_reeling.run_if(in_state(FishingState::ReelingUnhooked)),
                        is_fish_caught.run_if(in_state(FishingState::ReelingHooked)),
                        is_line_broken
                            .run_if(in_state(FishingState::ReelingHooked))
                            .after(is_fish_caught),
                        cast_line.run_if(in_state(FishingState::Casting)),
                        animate_fishing_line.run_if(not(in_state(FishingState::Casting))),
                    )
                        .after(bend_fishing_rod),
                    move_physics_objects
                        .after(is_fish_caught)
                        .after(is_line_broken),
                    animate_waves.after(is_line_broken),
                    adjust_fishing_line_size.after(animate_fishing_line),
                    draw_fishing_line.after(adjust_fishing_line_size),
                    animate_splash.after(cast_line),
                )
                    .run_if(in_state(CurrentInterface::Fishing)),
            )
            .add_systems(
                OnEnter(CurrentInterface::Fishing),
                (fishing_transition, switch_fishing_area),
            )
            .add_systems(OnExit(CurrentInterface::Fishing), overworld_transition)
            .add_systems(OnEnter(FishingState::Casting), begin_cast)
            .add_systems(
                OnTransition {
                    exited: FishingState::ReelingUnhooked,
                    entered: FishingState::Idle,
                },
                reset_interface,
            )
            .add_systems(OnEnter(MidnightState::Midnight), fishPopulation)
            .add_systems(
                OnTransition {
                    exited: FishingState::ReelingHooked,
                    entered: FishingState::Idle,
                },
                reset_interface,
            )
            .add_systems(
                Update,
                fish_update.run_if(in_state(CurrentInterface::Fishing)),
            );
    }
}

fn spawn_waves(
    commands: &mut Commands,
    texture: &Handle<Image>,
    layout: &Handle<TextureAtlasLayout>,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                texture: texture.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            TextureAtlas {
                layout: layout.clone(),
                index: 0,
            },
            Wave,
        ))
        .id()
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let waves_sheet_handle: Handle<Image> = asset_server.load("fishing_view/waves.png");
    let wave_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 4, 1, None, None);
    let wave_layout_handle = texture_atlases.add(wave_layout);
    let mut wave: Entity;

    //commands.insert_resource(FishBoundsDir {change_x: Vec3::new(0.,0.,0.), change_y: Vec3::new(0.,0.,0.)});

    commands.insert_resource(DirectionTimer {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });

    commands.insert_resource(ExclamationTimer {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
    });
    //let mut fish: HashMap<String, Species> = HashMap::new();

    //spawn example fish
    //BEMMY
    //BASS
    let cool_fish_handle: Handle<Image> = asset_server.load("fishing_view/awesome_fishy.png");
    commands.spawn((
        SpriteBundle {
            texture: cool_fish_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(320., 180.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(-8000., -8000., 901.),
                ..default()
            },
            ..default()
        },
        Fish {
            name: "bass",
            id: 0,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        InPond,
        BASS,
        Collision,
        MysteryFish,
        FishingLocal::Pond1,
        HungerCpt::new(BASS.time_of_day),
        HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
    ));

    commands.spawn((
        SpriteBundle {
            texture: cool_fish_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(320., 180.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.),
                ..default()
            },
            ..default()
        },
        Fish {
            name: "bass",
            id: 2,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        InPond,
        BASS,
        Collision,
        MysteryFish,
        FishingLocal::Pond2,
        HungerCpt::new(BASS.time_of_day),
        HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
    ));
    let fish_bass_handle: Handle<Image> = asset_server.load("fish/bass.png");
    wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);

    commands.spawn((
        SpriteBundle {
            texture: fish_bass_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(-8000., -8000., 901.),
                ..default()
            },
            ..default()
        },
        BASS,
        Fish {
            name: "Bass2",
            id: 2,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        PhysicsObject {
            mass: 2.0,
            position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 0.),
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            forces: Forces::default(),
            cd: BASS.cd,
            sa: (5.0 * 5.0, 5.0 * 8.0),
            waves: wave,
        },
        InPond,
        Collision,
        PhysicsFish,
        FishingLocal::Pond2,
        HungerCpt::new(BASS.time_of_day),
        HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
    ));

    //FISH BOX
    commands.spawn((
        SpriteBundle {
            texture: cool_fish_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(320., 180.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X - 40., FISHING_ROOM_Y + 40., 901.),
                ..default()
            },

            ..default()
        },
        Fish {
            name: "catfish",
            id: 1,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        InPond,
        CATFISH,
        Collision,
        MysteryFish,
        FishingLocal::Pond1,
        HungerCpt::new(CATFISH.time_of_day),
        HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
    ));

    let fish_bass_handle: Handle<Image> = asset_server.load("fish/bass.png");
    wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);

    commands.spawn((
        SpriteBundle {
            texture: fish_bass_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 901.),
                ..default()
            },
            ..default()
        },
        BASS,
        Fish {
            name: "Bass2",
            id: 0,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        PhysicsObject {
            mass: 2.0,
            position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 0.),
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            forces: Forces::default(),
            cd: BASS.cd,
            sa: (5.0 * 5.0, 5.0 * 8.0),
            waves: wave,
        },
        InPond,
        Collision,
        PhysicsFish,
        FishingLocal::Pond1,
        HungerCpt::new(BASS.time_of_day),
        HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
    ));

    let fish_bass_handle: Handle<Image> = asset_server.load("fish/catfish.png");
    wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);

    commands.spawn((
        SpriteBundle {
            texture: fish_bass_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 0.),
                ..default()
            },
            ..default()
        },
        CATFISH,
        Fish {
            name: "Catfish2",
            id: 1,
            is_caught: false,
            is_alive: true,
            touching_lure: false,
            length: 8.0,
            width: 5.0,
            weight: 2.0,
            time_of_day: (0, 12),
            weather: Weather::Sunny,
            depth: (0, 5),
            //x, y, z
            position: (8320, 3960),
            change_x: Vec3::new(0., 0., 0.),
            change_y: Vec3::new(0., 0., 0.),
            //length, width, depth
            bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
            age: 6.0,
            hunger: 1.0,
        },
        PhysicsObject {
            mass: 3.0,
            position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 0.),
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            forces: Forces::default(),
            cd: CATFISH.cd,
            sa: (5.0 * 5.0, 5.0 * 8.0),
            waves: wave,
        },
        InPond,
        Collision,
        PhysicsFish,
        FishingLocal::Pond1,
        HungerCpt::new(CATFISH.time_of_day),
        HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
    ));

    // HUD background
    commands.spawn((MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(208., 720.))),
        material: materials.add(Color::BLACK),
        transform: Transform {
            translation: Vec3::new(FISHING_ROOM_X + 536., FISHING_ROOM_Y, 999.25),
            ..default()
        },
        ..default()
    },));

    // HUD
    let hud_handle = asset_server.load("fishing_view/hud.png");

    commands.spawn((SpriteBundle {
        texture: hud_handle.clone(),
        transform: Transform {
            translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 999.75),
            ..default()
        },
        ..default()
    },));

    let fishing_sheet_handle: Handle<Image> = asset_server.load("fishing_view/pond_view.png");

    commands.spawn((
        SpriteBundle {
            texture: fishing_sheet_handle.clone(),
            sprite: Sprite { ..default() },
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 900.),
                ..default()
            },
            ..default()
        },
        PondScreen,
    ));

    let beach_sheet_handle: Handle<Image> = asset_server.load("fishing_view/beach_view.png");

    commands.spawn((
        SpriteBundle {
            texture: beach_sheet_handle.clone(),
            sprite: Sprite { ..default() },
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 900.),
                ..default()
            },
            ..default()
        },
        BeachScreen,
    ));

    //powerbar view
    let bar_sheet_handle = asset_server.load("fishing_view/power_bar.png");
    commands.spawn((
        SpriteBundle {
            texture: bar_sheet_handle.clone(),
            sprite: Sprite { ..default() },
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X + 575., FISHING_ROOM_Y - 308., 999.5),
                ..default()
            },
            ..default()
        },
        PowerBar { power: 0. },
    ));

    let player_fishing_handle = asset_server.load("fishing_view/back_fishing_sprite.png");
    commands.spawn((SpriteBundle {
        texture: player_fishing_handle.clone(),
        sprite: Sprite { ..default() },
        transform: Transform {
            translation: PLAYER_POSITION,
            ..default()
        },
        ..default()
    },));

    let exclamation_point_handle = asset_server.load("fishing_view/ExclamationPoint.png");
    commands.spawn((
        SpriteBundle {
            texture: exclamation_point_handle.clone(),

            sprite: Sprite { ..default() },

            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        exclam_point,
    ));

    let exclamation_point_handle = asset_server.load("fishing_view/ExclamationPoint.png");
    commands.spawn((
        SpriteBundle {
            texture: exclamation_point_handle.clone(),

            sprite: Sprite { ..default() },

            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        exclam_point,
    ));

    // Fishing rod
    let default_rod_type = &FishingRodType::NORMAL;
    let segment_count: usize = (FishingRodType::NORMAL.length / BENDING_RESOLUTION) as usize;

    let mut rod_info: FishingRod = FishingRod {
        rod_type: default_rod_type,
        rotation: PI / 2.,
        material: materials.add(default_rod_type.blank_color),
        segments: Vec::with_capacity(segment_count),
        tip_pos: Vec3::new(
            PLAYER_POSITION.x,
            PLAYER_POSITION.y + default_rod_type.length * PIXELS_PER_METER,
            0.,
        ),
    };

    // Fishling line
    for i in (0..segment_count).rev() {
        let l = i as f32 * BENDING_RESOLUTION;
        let radius = default_rod_type.thickness * l / default_rod_type.length;
        let radius_pixels = (radius * 750.).max(1.);

        let segment = commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(radius_pixels, radius_pixels))),
                material: rod_info.material.clone(),
                ..default()
            },
            FishingRodSegment,
        ));

        rod_info.segments.push(segment.id());
    }

    let mut particle_info: ParticleList = ParticleList {
        particle_list: Vec::with_capacity(PARTICLECOUNT),
        ..default()
    };

    for i in 0..PARTICLECOUNT {
        let particlepos = Vec3::new(0., 0., 0.);
        particle_info
            .particle_list
            .push(Particle::new(particlepos, Vec3::new(0., 0., 0.), 10.));
    }

    commands.spawn(particle_info);

    let fishing_rod_handle = asset_server.load(default_rod_type.texture);

    commands.spawn((
        SpriteBundle {
            texture: fishing_rod_handle.clone(),
            transform: Transform {
                translation: Vec3::new(PLAYER_POSITION.x, PLAYER_POSITION.y, 901.),
                ..default()
            },
            ..default()
        },
        rod_info,
    ));

    commands.spawn(FishingLine::new(&FishingLineType::MONOFILILMENT));

    let splashes_sheet_handle: Handle<Image> = asset_server.load("fishing_view/splashes.png");
    let splash_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 3, 1, None, None);
    let splash_layout_len = splash_layout.textures.len();
    let splash_layout_handle = texture_atlases.add(splash_layout);
    commands.spawn((
        SpriteBundle {
            texture: splashes_sheet_handle.clone(),
            transform: Transform::from_xyz(
                FISHING_ROOM_X - 90.,
                FISHING_ROOM_Y - (WIN_H / 2.) + 100.,
                930.,
            ),
            visibility: Visibility::Hidden,
            ..default()
        },
        TextureAtlas {
            layout: splash_layout_handle.clone(),
            index: 0,
        },
        AnimationTimer::new(0.2),
        AnimationFrameCount(splash_layout_len), //number of different frames that we have
        Splash::default(),
    ));

    let lure_sheet_handle: Handle<Image> = asset_server.load("lures/baits.png");
    let lure_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 3, 1, None, None);
    let lure_layout_len = lure_layout.textures.len();
    let lure_layout_handle = texture_atlases.add(lure_layout);

    // Lure display on HUD
    commands.spawn((
        SpriteBundle {
            texture: lure_sheet_handle.clone(),
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X + 545., FISHING_ROOM_Y + 255., 999.8),
                scale: (Vec3::splat(3.0)),
                ..default()
            },
            visibility: Visibility::Visible,
            ..default()
        },
        TextureAtlas {
            layout: lure_layout_handle.clone(),
            index: 0,
        },
        LureHUD,
        AnimationFrameCount(lure_layout_len),
        AnimationTimer::new(0.2), //number of different frames that we have
    ));

    // Physical lure
    wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);

    commands.spawn((
        SpriteBundle {
            texture: lure_sheet_handle.clone(),
            transform: Transform::from_xyz(
                FISHING_ROOM_X - 90.,
                FISHING_ROOM_Y - (WIN_H / 2.) + 100.,
                930.,
            ),
            visibility: Visibility::Hidden,
            ..default()
        },
        TextureAtlas {
            layout: lure_layout_handle.clone(),
            index: 0,
        },
        PhysicsObject {
            mass: 2.0,
            position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y + 100., 0.),
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            forces: Forces::default(),
            cd: Lure::BOBBER.cd,
            sa: Lure::BOBBER.sa,
            waves: wave,
        },
        Collision,
        Lure::BOBBER,
    ));

    //spawning in the lilypad
    let lily_sheet_handle: Handle<Image> = asset_server.load("fishing_view/lilypad.png");
    let deep_sheet_handle: Handle<Image> = asset_server.load("fishing_view/deep.png");

    commands.spawn((
        SpriteBundle {
            texture: lily_sheet_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(128., 128.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X + 160., FISHING_ROOM_Y + 100., 901.),
                ..default()
            },
            ..default()
        },
        Collision,
        PondObstruction,
        ObstType::Pad,
        InPond,
        FishingLocal::Pond1,
    ));

    commands.spawn((
        SpriteBundle {
            texture: lily_sheet_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(128., 128.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-8000., -8000., 901.),
                ..default()
            },
            ..default()
        },
        Collision,
        PondObstruction,
        ObstType::Pad,
        InPond,
        FishingLocal::Pond2,
    ));

    let debris_sheet_handle: Handle<Image> = asset_server.load("fishing_view/water_bottle.png");
    let bush_debris_sheet_handle: Handle<Image> = asset_server.load("tiles/bush_no_shadow.png");

    commands.spawn((
        SpriteBundle {
            texture: debris_sheet_handle.clone(),
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X + 100., FISHING_ROOM_Y + 50., 901.),
                ..default()
            },
            ..default()
        },
        PondObstruction,
        ObstType::Debris,
        DebrisType::WATER_BOTTLE,
        DebrisHooked { hooked: false },
        InPond,
        FishingLocal::Pond1,
    ));

    commands.spawn((
        SpriteBundle {
            texture: bush_debris_sheet_handle.clone(),
            transform: Transform {
                translation: Vec3::new(FISHING_ROOM_X - 500., FISHING_ROOM_Y - 1000., 901.),
                ..default()
            },
            visibility: Visibility::Visible,
            ..default()
        },
        Collision,
        PondObstruction,
        ObstType::Debris,
        DebrisType::BUSH,
        DebrisHooked { hooked: false },
        InPond,
        FishingLocal::Pond1,
    ));

    commands.spawn((
        SpriteBundle {
            texture: debris_sheet_handle.clone(),
            sprite: Sprite { ..default() },
            visibility: Visibility::Visible,
            transform: Transform {
                translation: Vec3::new(-8000., -8000., 901.),
                ..default()
            },
            ..default()
        },
        PondObstruction,
        ObstType::Debris,
        DebrisType::WATER_BOTTLE,
        InPond,
        FishingLocal::Pond2,
    ));

    commands.spawn((
        SpriteBundle {
            texture: debris_sheet_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-8000., -8000., 901.),
                ..default()
            },
            ..default()
        },
        PondObstruction,
        ObstType::Debris,
        DebrisType::WATER_BOTTLE,
        InPond,
        FishingLocal::Ocean,
    ));

    commands.spawn((
        SpriteBundle {
            texture: deep_sheet_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(128., 128.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-9000., -9000., 901.),
                ..default()
            },
            ..default()
        },
        Collision,
        PondObstruction,
        ObstType::Fissure,
        InPond,
        FishingLocal::Pond1,
    ));
}

fn move_fish(
    mut fish_details: Query<
        (&mut Fish, &mut Transform, &Species, &mut FishingLocal),
        (
            With<InPond>,
            With<Collision>,
            Without<PhysicsObject>,
            Without<PondObstruction>,
        ),
    >,
    mut obst_details: Query<
        (&mut Transform, &mut ObstType, &mut FishingLocal),
        (
            With<PondObstruction>,
            With<Collision>,
            With<InPond>,
            Without<FishDetails>,
        ),
    >,
    time: Res<Time>,
    mut config: ResMut<DirectionTimer>,
    //mut fish_direction: ResMut<FishBoundsDir>
) {
    let mut rng = rand::thread_rng();
    let mut obst_rng = rand::thread_rng();
    config.timer.tick(time.delta());
    //let mut obst_details = obst_details.single_mut();

    //let mut rng = rand::thread_rng();

    for (mut fish_details, mut fish_pos, fish_species, fishLoc) in fish_details.iter_mut() {
        //let mut rng = rand::thread_rng();

        //move towards the obsticle on the x bounds

        if config.timer.finished() {
            let move_type: i32 = rng.gen_range(0..9);
            let dir: i32 = obst_rng.gen_range(0..9);
            let mut move_skew: i32 = 0;
            //finding where to go in relation to the
            //position in relation to x row
            for (obst_details, obstical_type, obstLoc) in obst_details.iter_mut() {
                //go back and account for margin of error done
                if *obstical_type == fish_species.obj_pref.0 {
                    //if fish_details.name == "catfish"{
                    if *obstLoc == *fishLoc {
                        move_skew = fish_species.obj_pref.1;
                        if obst_details.translation.x >= fish_pos.translation.x {
                            fish_details.change_x = Vec3::new(0.5, 0., 0.);
                        } else if obst_details.translation.x < fish_pos.translation.x {
                            fish_details.change_x = Vec3::new(-0.5, 0., 0.);
                        }

                        //move towards the obsticle on the right bounds
                        if obst_details.translation.y >= fish_pos.translation.y {
                            fish_details.change_y = Vec3::new(0., 0.5, 0.);
                        } else if obst_details.translation.y < fish_pos.translation.y {
                            fish_details.change_y = Vec3::new(0.0, -0.5, 0.);
                        }
                    }
                    //}
                }
                // else if *obstical_type == ObstType::Pad{
                //     if fish_details.name == "bass"{
                //         if obst_details.translation.x >= fish_pos.translation.x{
                //             fish_details.change_x = Vec3::new(0.5, 0., 0.);

                //         }
                //         else if obst_details.translation.x < fish_pos.translation.x{
                //             fish_details.change_x = Vec3::new(-0.5, 0., 0.);
                //         }

                //         //move towards the obsticle on the right bounds
                //         if obst_details.translation.y >= fish_pos.translation.y{
                //             fish_details.change_y = Vec3::new(0., 0.5, 0.);

                //         }
                //         else if obst_details.translation.y < fish_pos.translation.y{
                //             fish_details.change_y = Vec3::new(0.0, -0.5, 0.);
                //         }
                //     }
                // }
                //for each collision object add a
            }

            println!("timer finished");

            //println!("numer is {} {:?}", dir, fish_details.name);
            if move_type >= 4 + move_skew {
                if dir == 0 {
                    fish_details.change_x = Vec3::new(0., 0., 0.);
                    fish_details.change_y = Vec3::new(0., 0.5, 0.);
                } else if dir == 1 {
                    fish_details.change_x = Vec3::new(0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., 0.5, 0.);
                } else if dir == 2 {
                    fish_details.change_x = Vec3::new(0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., 0., 0.);
                } else if dir == 3 {
                    fish_details.change_x = Vec3::new(0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., -0.5, 0.);
                } else if dir == 4 {
                    fish_details.change_x = Vec3::new(0., 0., 0.);
                    fish_details.change_y = Vec3::new(0., -0.5, 0.);
                } else if dir == 5 {
                    fish_details.change_x = Vec3::new(-0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., -0.5, 0.);
                } else if dir == 6 {
                    fish_details.change_x = Vec3::new(-0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., 0., 0.);
                } else if dir == 7 {
                    fish_details.change_x = Vec3::new(-0.5, 0., 0.);
                    fish_details.change_y = Vec3::new(0., 0.5, 0.);
                }
            } else {
                //println!("moving toward the object");
                //if it isnt moving in a random direction,
            }
        }

        //match it then set up the vector for the next tree seconds, keep the stuff about borders

        //println!("fish pos x {}, fish pos y {}", change_x, change_y);
        //CHANGE THESE TO CONSTANTS LATER!!!!!
        /*

        //pub const FISHINGROOMX: f32 = 8960.;
        //pub const FISHINGROOMY: f32 = 3600.;

                 */

        let holdx: Vec3 = fish_pos.translation + fish_details.change_x;
        if (holdx.x) >= (-640. + 160.) && (holdx.x) <= (431. - 160.) {
            //println!("{:?}", fish_pos.translation);
            fish_pos.translation += fish_details.change_x;
        } else {
            // println!("fish: {:?} {:?}", fish_details.name, fish_details.id);
            // println!("holdx = {:?}", holdx);
        }
        let holdy: Vec3 = fish_pos.translation + fish_details.change_y;
        if (holdy.y) >= (-1400. - 224. + 90.) && (holdy.y) <= (-1400. + 360. - 90.) {
            //println!("fish going up");
            fish_pos.translation += fish_details.change_y;
        } else {
            //println!("{},  {}", FISHING_ROOM_X, FISHING_ROOM_Y);
            //println!(
            //"fish: {:?} {:?} {:?}",
            //  fish_details.name, fish_details.id, fish_pos.translation
            //);
            // println!("holdx = {:?}", holdy);
        }
    }
    //fish_pos.translation += change_y;
    //return (self.position.0 + x, self.position.1+y)
}

//function to poplulate

fn switch_fishing_area(
    mut commands: Commands,
    mut fish_details: Query<
        (
            &mut Fish,
            &Species,
            &mut Transform,
            &mut Visibility,
            &FishingLocal,
        ),
        (
            With<InPond>,
            With<Fish>,
            With<Collision>,
            With<MysteryFish>,
            Without<PhysicsObject>,
            Without<Lure>,
        ),
    >,
    mut backgroundDeetsPond: Query<
        &mut Transform,
        (
            Without<BeachScreen>,
            Without<Collision>,
            Without<PhysicsObject>,
            Without<Lure>,
            With<PondScreen>,
            Without<MysteryFish>,
            Without<InPond>,
        ),
    >,
    mut backgroundDeetsBeach: Query<
        &mut Transform,
        (
            With<BeachScreen>,
            Without<Collision>,
            Without<PhysicsObject>,
            Without<Lure>,
            Without<PondScreen>,
            Without<MysteryFish>,
            Without<InPond>,
        ),
    >,
    mut fishes_phys: Query<
        (Entity, &mut Transform, &FishingLocal),
        (
            With<PhysicsFish>,
            With<Fish>,
            With<Collision>,
            With<InPond>,
            With<PhysicsObject>,
            Without<Lure>,
            Without<MysteryFish>,
            Without<PondScreen>,
            Without<BeachScreen>,
        ),
    >,
    mut obst_details: Query<
        (&mut Transform, &mut ObstType, &FishingLocal),
        (
            With<PondObstruction>,
            With<Collision>,
            With<InPond>,
            Without<FishDetails>,
            Without<MysteryFish>,
            Without<PhysicsObject>,
            Without<Lure>,
            Without<PondScreen>,
            Without<BeachScreen>,
        ),
    >,
    state: Res<State<FishingLocal>>,
) {
    let mut beachScr = backgroundDeetsBeach.single_mut();
    let mut pondScr = backgroundDeetsPond.single_mut();

    if state.eq(&FishingLocal::Pond1) {
        for (mut fish, species, mut transform, mut visibility, loc) in &mut fish_details {
            if *loc == FishingLocal::Pond1 {
                transform.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                transform.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut obstPos, obstType, obstLoc) in &mut obst_details {
            if *obstLoc == FishingLocal::Pond1 {
                if *obstType == ObstType::Pad {
                    println!("spawning pad in");
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X + 160., FISHING_ROOM_Y + 100., 901.);
                } else if *obstType == ObstType::Fissure {
                    println!("spawning fissure in");

                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X - 300., FISHING_ROOM_Y - 100., 901.);
                } else if *obstType == ObstType::Debris {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X - 500., FISHING_ROOM_Y + 50., 901.);
                }
            } else {
                obstPos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut ent, mut pos, location) in &mut fishes_phys {
            if *location == FishingLocal::Pond1 {
                pos.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                pos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        //POND BEACH
        pondScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 900.);
        beachScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 100.);
    }
    if state.eq(&FishingLocal::Pond2) {
        for (mut fish, species, mut transform, mut visibility, loc) in &mut fish_details {
            if *loc == FishingLocal::Pond2 {
                transform.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                transform.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut obstPos, obstType, obstLoc) in &mut obst_details {
            if *obstLoc == FishingLocal::Pond2 {
                if *obstType == ObstType::Pad {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X - 160., FISHING_ROOM_Y + 300., 901.);
                } else if *obstType == ObstType::Fissure {
                    obstPos.translation = Vec3::new(FISHING_ROOM_X + 260., FISHING_ROOM_Y, 901.);
                } else if *obstType == ObstType::Debris {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X - 350., FISHING_ROOM_Y + 200., 901.);
                }
            } else {
                obstPos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut ent, mut pos, location) in &mut fishes_phys {
            if *location == FishingLocal::Pond2 {
                pos.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                pos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }

        //POND BEACH
        pondScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 900.);
        beachScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 100.);
    }
    if state.eq(&FishingLocal::Ocean) {
        for (mut fish, species, mut transform, mut visibility, loc) in &mut fish_details {
            if *loc == FishingLocal::Ocean {
                transform.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                transform.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut obstPos, obstType, obstLoc) in &mut obst_details {
            if *obstLoc == FishingLocal::Ocean {
                if *obstType == ObstType::Pad {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X + 160., FISHING_ROOM_Y + 100., 901.);
                } else if *obstType == ObstType::Fissure {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X - 360., FISHING_ROOM_Y - 100., 901.);
                } else if *obstType == ObstType::Debris {
                    obstPos.translation =
                        Vec3::new(FISHING_ROOM_X + 260., FISHING_ROOM_Y + 300., 901.);
                }
            } else {
                obstPos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }
        for (mut ent, mut pos, location) in &mut fishes_phys {
            if *location == FishingLocal::Ocean {
                pos.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 901.);
            } else {
                pos.translation = Vec3::new(-8000., -8000., 901.);
            }
        }

        //POND BEACH
        pondScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 100.);
        beachScr.translation = Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 900.);
    }

    //check what fishing state youre going into, check if pond one move pond 1 fish in, move pond 2 fish out
}

//FISHPONDADD
fn fishPopulation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut fish_details: Query<
        (
            &mut Fish,
            &Species,
            &mut Transform,
            &mut Visibility,
            &FishingLocal,
        ),
        (
            With<InPond>,
            With<Fish>,
            With<Collision>,
            With<MysteryFish>,
            Without<PhysicsObject>,
            Without<Bobber>,
        ),
    >,
) {
    let waves_sheet_handle: Handle<Image> = asset_server.load("fishing_view/waves.png");
    let wave_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 4, 1, None, None);
    let wave_layout_handle = texture_atlases.add(wave_layout);
    let mut wave: Entity;

    let target_p1 = 10;
    let target_p2 = 10;
    let target_beach = 10;

    let mut curr_p1 = 0;
    let mut curr_p2 = 0;
    let mut curr_beach = 0;

    let mut total_fish = 3;

    let fish_bass_handle: Handle<Image> = asset_server.load("fish/bass.png");
    let fish_cat_handle: Handle<Image> = asset_server.load("fish/catfish.png");
    let fish_mahimahi_handle: Handle<Image> = asset_server.load("fish/mahimahi.png");
    let fish_swordfish_handle: Handle<Image> = asset_server.load("fish/swordfish.png");
    let fish_handfish_handle: Handle<Image> = asset_server.load("fish/redhandfish.png");
    let fish_tuna_handle: Handle<Image> = asset_server.load("fish/tuna.png");
    let cool_fish_handle: Handle<Image> = asset_server.load("fishing_view/awesome_fishy.png");

    let mut rng = rand::thread_rng();
    //let move_type: i32 = rng.gen_range(0..99);

    for (fish, spec, transform, vis, loc) in fish_details.iter_mut() {
        if *loc == FishingLocal::Pond1 {
            curr_p1 += 1;
        }
        if *loc == FishingLocal::Pond2 {
            curr_p2 += 1;
        }
        if *loc == FishingLocal::Ocean {
            curr_beach += 1;
        }
    }
    /*
    BASS.length.0;
    BASS.length.1;
    */
    total_fish = curr_p1 + curr_p2 + curr_beach;
    //println!("there are {} p1 fish {} p2 fish and {} beach Fish", curr_p1, curr_p2, curr_beach);
    //p1
    let mut p1Odds = target_p1 - curr_p1;
    for w in 0..3 {
        //random check 1-100
        //if less than p1Odds
        //if it hits
        let rand_check: i32 = rng.gen_range(0..11);

        if rand_check <= p1Odds {
            curr_p1 += 1;
            p1Odds = target_p1 - curr_p1;
            println!("adding p1 fish");
            total_fish += 1;

            if rng.gen_range(0..100) <= 50 {
                //spawning bass
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                let fish_length = rng.gen_range(BASS.length.0..BASS.length.1);
                let fish_width = rng.gen_range(BASS.width.0..BASS.width.1);
                let fish_weight = rng.gen_range(BASS.weight.0..BASS.weight.1);

                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "bass",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: (0, 5),
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    BASS,
                    Collision,
                    MysteryFish,
                    FishingLocal::Pond1,
                    HungerCpt::new(BASS.time_of_day),
                    HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: fish_bass_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    BASS,
                    Fish {
                        name: "Bass2",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: BASS.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 2.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: BASS.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Pond1,
                    HungerCpt::new(BASS.time_of_day),
                    HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
                ));
            } else {
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                let fish_length = rng.gen_range(CATFISH.length.0..CATFISH.length.1);
                let fish_width = rng.gen_range(CATFISH.width.0..CATFISH.width.1);
                let fish_weight = rng.gen_range(CATFISH.weight.0..CATFISH.weight.1);

                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "catfish",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 18),
                        weather: Weather::Rainy,
                        depth: CATFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    CATFISH,
                    Collision,
                    MysteryFish,
                    FishingLocal::Pond1,
                    HungerCpt::new(CATFISH.time_of_day),
                    HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: fish_cat_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    CATFISH,
                    Fish {
                        name: "CATFISH",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: CATFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 5.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: CATFISH.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Pond1,
                    HungerCpt::new(CATFISH.time_of_day),
                    HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
                ));
                //spawning catfish
            }
        } else {
            println!("no p1 fish");
        }
        //if it doesnt run again
    }

    //p2
    let mut p2Odds = target_p2 - curr_p2;
    for i in 0..3 {
        let rand_check: i32 = rng.gen_range(0..11);

        if rand_check <= p2Odds {
            curr_p2 += 1;
            p2Odds = target_p2 - curr_p2;
            println!("adding p2 fish");
            total_fish += 1;

            if rng.gen_range(0..100) <= 50 {
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                //spawning bass
                let fish_length = rng.gen_range(BASS.length.0..BASS.length.1);
                let fish_width = rng.gen_range(BASS.width.0..BASS.width.1);
                let fish_weight = rng.gen_range(BASS.weight.0..BASS.weight.1);

                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "bass",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: (0, 5),
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    BASS,
                    Collision,
                    MysteryFish,
                    FishingLocal::Pond2,
                    HungerCpt::new(BASS.time_of_day),
                    HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: fish_bass_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    BASS,
                    Fish {
                        name: "Bass2",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: BASS.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 1.0,
                    },
                    PhysicsObject {
                        mass: 2.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: BASS.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Pond2,
                    HungerCpt::new(BASS.time_of_day),
                    HookProbCpt::new(BASS.time_of_day, BASS.depth, BASS.catch_prob),
                ));
            } else {
                let fish_length = rng.gen_range(CATFISH.length.0..CATFISH.length.1);
                let fish_width = rng.gen_range(CATFISH.width.0..CATFISH.width.1);
                let fish_weight = rng.gen_range(CATFISH.weight.0..CATFISH.weight.1);
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "catfish",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 18),
                        weather: Weather::Rainy,
                        depth: CATFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 1.0,
                    },
                    InPond,
                    CATFISH,
                    Collision,
                    MysteryFish,
                    FishingLocal::Pond2,
                    HungerCpt::new(CATFISH.time_of_day),
                    HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: fish_cat_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    CATFISH,
                    Fish {
                        name: "CATFISH",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: CATFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 5.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: CATFISH.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Pond2,
                    HungerCpt::new(CATFISH.time_of_day),
                    HookProbCpt::new(CATFISH.time_of_day, CATFISH.depth, CATFISH.catch_prob),
                ));
            }
        } else {
            println!("no p2 fish");
        }
    }

    //beach
    let mut beachOdds = target_beach - curr_beach;
    for n in 0..3 {
        let rand_check: i32 = rng.gen_range(0..11);

        if rand_check <= beachOdds {
            curr_beach += 1;
            beachOdds = target_beach - curr_beach;
            println!("adding p2 fish");
            total_fish += 1;
            let fish_num = rng.gen_range(0..100);
            if fish_num <= 30 {
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                //spawning bass
                let fish_length = rng.gen_range(MAHIMAHI.length.0..MAHIMAHI.length.1);
                let fish_width = rng.gen_range(MAHIMAHI.width.0..MAHIMAHI.width.1);
                let fish_weight = rng.gen_range(MAHIMAHI.weight.0..MAHIMAHI.weight.1);

                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "MAHI",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Thunderstorm,
                        depth: (0, 5),
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    MAHIMAHI,
                    Collision,
                    MysteryFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(MAHIMAHI.time_of_day),
                    HookProbCpt::new(MAHIMAHI.time_of_day, MAHIMAHI.depth, MAHIMAHI.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: fish_mahimahi_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    MAHIMAHI,
                    Fish {
                        name: "MAHI",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: MAHIMAHI.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 2.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: MAHIMAHI.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave, //cd
                                     //forces
                                     //waves

                                     /*
                                                             pub struct PhysicsObject {
                                         pub mass: f32,
                                         pub position: Vec3,
                                         pub rotation: Vec3,
                                         pub velocity: Vec3,
                                         pub forces: Forces,
                                         pub cd: (f32, f32),
                                         pub sa: (f32, f32),
                                         pub waves: Entity
                                     } */
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(MAHIMAHI.time_of_day),
                    HookProbCpt::new(MAHIMAHI.time_of_day, MAHIMAHI.depth, MAHIMAHI.catch_prob),
                ));
            } else if fish_num > 30 && fish_num <= 60 {
                let fish_length = rng.gen_range(TUNA.length.0..TUNA.length.1);
                let fish_width = rng.gen_range(TUNA.width.0..TUNA.width.1);
                let fish_weight = rng.gen_range(TUNA.weight.0..TUNA.weight.1);
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "Tuna",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 18),
                        weather: Weather::Thunderstorm,
                        depth: TUNA.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 1.0,
                    },
                    InPond,
                    TUNA,
                    Collision,
                    MysteryFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(TUNA.time_of_day),
                    HookProbCpt::new(TUNA.time_of_day, TUNA.depth, TUNA.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        //fix
                        texture: fish_tuna_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    TUNA,
                    Fish {
                        name: "Tuna",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: TUNA.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 1.0,
                    },
                    PhysicsObject {
                        mass: 10.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: TUNA.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(TUNA.time_of_day),
                    HookProbCpt::new(TUNA.time_of_day, TUNA.depth, TUNA.catch_prob),
                ));
            } else if fish_num >= 60 && fish_num < 90 {
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                let fish_length = rng.gen_range(SWORDFISH.length.0..SWORDFISH.length.1);
                let fish_width = rng.gen_range(SWORDFISH.width.0..SWORDFISH.width.1);
                let fish_weight = rng.gen_range(SWORDFISH.weight.0..SWORDFISH.weight.1);

                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "SWORDFISH",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 18),
                        weather: Weather::Thunderstorm,
                        depth: SWORDFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    SWORDFISH,
                    Collision,
                    MysteryFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(SWORDFISH.time_of_day),
                    HookProbCpt::new(SWORDFISH.time_of_day, SWORDFISH.depth, SWORDFISH.catch_prob),
                ));
                commands.spawn((
                    SpriteBundle {
                        //fix
                        texture: fish_swordfish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    SWORDFISH,
                    Fish {
                        name: "SWORDFISH",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: SWORDFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 10.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: SWORDFISH.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(SWORDFISH.time_of_day),
                    HookProbCpt::new(SWORDFISH.time_of_day, SWORDFISH.depth, SWORDFISH.catch_prob),
                ));
            } else if fish_num >= 90 {
                wave = spawn_waves(&mut commands, &waves_sheet_handle, &wave_layout_handle);
                let fish_length = rng.gen_range(REDHANDFISH.length.0..REDHANDFISH.length.1);
                let fish_width = rng.gen_range(REDHANDFISH.width.0..REDHANDFISH.width.1);
                let fish_weight = rng.gen_range(REDHANDFISH.weight.0..REDHANDFISH.weight.1);
                println!("spawning legendary fish");
                commands.spawn((
                    SpriteBundle {
                        texture: cool_fish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(320., 180.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    Fish {
                        name: "Handfish",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 18),
                        weather: Weather::Sunny,
                        depth: REDHANDFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    InPond,
                    REDHANDFISH,
                    Collision,
                    MysteryFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(REDHANDFISH.time_of_day),
                    HookProbCpt::new(
                        REDHANDFISH.time_of_day,
                        REDHANDFISH.depth,
                        REDHANDFISH.catch_prob,
                    ),
                ));
                commands.spawn((
                    SpriteBundle {
                        //fix
                        texture: fish_handfish_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        transform: Transform {
                            translation: Vec3::new(-8000., -8000., 901.),
                            ..default()
                        },
                        ..default()
                    },
                    REDHANDFISH,
                    Fish {
                        name: "SWORDFISH",
                        id: total_fish as u32,
                        is_caught: false,
                        is_alive: true,
                        touching_lure: false,
                        length: fish_length,
                        width: fish_width,
                        weight: fish_weight,
                        time_of_day: (0, 12),
                        weather: Weather::Sunny,
                        depth: REDHANDFISH.depth,
                        //x, y, z
                        position: (8320, 3960),
                        change_x: Vec3::new(0., 0., 0.),
                        change_y: Vec3::new(0., 0., 0.),
                        //length, width, depth
                        bounds: (FISHING_ROOM_X as i32 + 100, FISHING_ROOM_Y as i32 + 100),
                        age: 1.0,
                        hunger: 10.0,
                    },
                    PhysicsObject {
                        mass: 10.0,
                        position: Vec3::new(FISHING_ROOM_X, FISHING_ROOM_Y, 0.),
                        rotation: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        forces: Forces::default(),
                        cd: REDHANDFISH.cd,
                        sa: (5.0 * 5.0, 5.0 * 8.0),
                        waves: wave,
                    },
                    InPond,
                    Collision,
                    PhysicsFish,
                    FishingLocal::Ocean,
                    HungerCpt::new(REDHANDFISH.time_of_day),
                    HookProbCpt::new(
                        REDHANDFISH.time_of_day,
                        REDHANDFISH.depth,
                        REDHANDFISH.catch_prob,
                    ),
                ));
            }
        } else {
            println!("no beach fish");
        }
    }

    //this is working lets start adding stuff/ look at psuedo code.

    //pond section and river section.
    println!("ITS 23!!! adding new fish now!!!");
}

fn fish_area_lure(
    mut commands: Commands,
    mut fish_details: Query<
        (
            &mut Fish,
            &Species,
            &mut Transform,
            &mut Visibility,
            &HookProbCpt,
        ),
        (
            With<InPond>,
            With<Fish>,
            With<Collision>,
            With<MysteryFish>,
            Without<PhysicsObject>,
            Without<Lure>,
        ),
    >,
    mut lure: Query<
        (
            &Transform,
            Entity,
            &mut PhysicsObject,
            &mut Visibility,
            &Lure,
        ),
        (
            With<Lure>,
            With<PhysicsObject>,
            Without<Fish>,
            Without<MysteryFish>,
        ),
    >,
    mut fishes: Query<
        (
            Entity,
            &mut Fish,
            &Species,
            &mut PhysicsObject,
            &mut Transform,
            &mut Visibility,
        ),
        (
            With<PhysicsFish>,
            With<Fish>,
            With<Collision>,
            With<InPond>,
            With<PhysicsObject>,
            Without<Lure>,
            Without<MysteryFish>,
        ),
    >, //add this in as the fish query, change the position of it at the end
    mut exclamation: Query<
        (&mut Transform, &mut Visibility),
        (
            With<exclam_point>,
            Without<InPond>,
            Without<Lure>,
            Without<PhysicsFish>,
        ),
    >,
    //mut fishes_physics: Query<(Entity, &Fish, &Species, &mut PhysicsObject), (With<Fish>, Without<Lure>)>,
    weather: Res<WeatherState>,
    region: Res<State<Region>>,
    timer: Res<GameDayTimer>,
    mut prob_timer: ResMut<ProbTimer>,
    mut next_state: ResMut<NextState<FishingState>>,
    time: Res<Time>,
    mut config: ResMut<ExclamationTimer>,
    debris_details: Query<(&DebrisType, &DebrisHooked)>,
) {
    let (lure_transform, lure_entity_id, mut lure_physics, mut lure_vis, lure_details) =
        lure.single_mut();
    let lure_position = lure_transform.translation;
    //let (bob, tile) = lure.single_mut();
    //let (bob, tile, mut lure_vis) = lure.single_mut();
    //let (mut exclam_transform, mut exclam_vis) = exclamation.single_mut();

    for (mut fish_details, fish_species, fish_pos, mut fish_vis, hook_cpt) in
        fish_details.iter_mut()
    {
        let fish_pos_loc = fish_pos.translation;
        let lure_position = lure_transform.translation;

        //println!("fish {:?} {} x {} y \n lure:  {} x {} y ", fishes_details.name, fish_pos.translation.x, fish_pos.translation.y, lure_position.x, lure_position.y);

        //let lure_position = bob.translation;

        if fish_pos_loc.y - 180. / 2. > lure_position.y + 50.
            || fish_pos_loc.y + 180. / 2. < lure_position.y - 50.
            || fish_pos_loc.x + 320. / 2. < lure_position.x - 50.
            || fish_pos_loc.x - 320. / 2. > lure_position.x + 50.
        {
            //there is no hit
            fish_details.touching_lure = false;
            //println!("fish {:?}, {:?}", fishes_details.name, fish_pos.translation);
            //println!("no hit");
            continue;
        }
        fish_details.touching_lure = true;
        //println!("fish {:?}, {:?}", fish_details.name, fish_pos.translation);
        //println!("lure hit");

        //let (entity_id, mut fishy_details, fish_species, mut fish_physics, mut fishy_transform, mut fishy_vis) = fishes.single_mut();

        //ERROR HERE
        if hook_fish(
            (&mut fish_details, fish_species, hook_cpt),
            &weather,
            &region,
            &timer,
            &mut prob_timer,
            &time,
            lure_details,
        ) {
            for (
                entity_id,
                mut fishy_details,
                fish_species,
                mut fish_physics,
                mut fishy_transform,
                mut fishy_vis,
            ) in fishes.iter_mut()
            {
                if fishy_details.id == fish_details.id {
                    //fish number matches the other number of the caught fish
                    println!("FIRST: {:?}", fishy_transform.translation);
                    //println!("FIRST: {:?}", exclam_transform.translation);
                    fishy_transform.translation = lure_position;
                    //exclam_transform.translation = lure_position;
                    //exclam_transform.translation.y += 40.;
                    //*exclam_vis = Visibility::Visible;

                    config.timer.tick(time.delta());

                    /*if config.timer.finished()
                    {
                        println!("hiding it");
                        *exclam_vis = Visibility::Hidden;
                    }*/

                    //println!("SECOND: {:?}", exclam_transform.translation);
                    fish_physics.position = lure_physics.position;
                    //fishy_transform.translation = fish_physics.position.with_z(901.);
                    *lure_vis = Visibility::Hidden; //yes
                    *fish_vis = Visibility::Hidden; //yes
                    *fishy_vis = Visibility::Visible;
                    fishy_details.is_caught = true;
                    //println!("FIRST: {:?}", fishy_transform.translation);

                    println!("SECOND: {:?}", fishy_transform.translation);
                    //for (physics_object, mut transform) in objects.iter_mut() {
                    //transform.translation = physics_object.position.with_z(901.);
                    //unhide the actual fish
                    fish_physics.mass = fish_physics.mass + lure_physics.mass; //yes

                    for (debris_info, debris_hooked) in debris_details.iter() {
                        if debris_hooked.hooked {
                            fish_physics.cd = (
                                fish_physics.cd.0 + debris_info.drag_increase,
                                fish_physics.cd.1 + debris_info.drag_increase,
                            );
                        }
                    }

                    println!("fish name {:?}", fishy_details.name);
                    commands.entity(lure_entity_id).remove::<Hooked>(); //yes
                    commands.entity(entity_id).insert(Hooked); //yes
                    next_state.set(FishingState::ReelingHooked);

                    break;
                } else {
                    println!("wrong fish this is a {:?}", fish_details.name);
                }
            }
            //next_state.set(FishingState::ReelingHooked); //yes
            //break; //yes
        }
    }
}

fn fishing_transition(
    mut return_pos: ResMut<PlayerReturnPos>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut power_bar: Query<(&mut Transform, &mut PowerBar), (With<PowerBar>, Without<Camera>)>,
    mut rod: Query<&mut Transform, (With<FishingRod>, Without<Camera>, Without<PowerBar>)>,
    player_inventory: Query<&mut PlayerInventory>,
    mut fishes: Query<(&mut Visibility), (With<MysteryFish>, With<InPond>)>,
) {
    let mut camera_transform = camera.single_mut();
    let (mut power_bar_transform, mut power) = power_bar.single_mut();
    let mut rod_transform = rod.single_mut();
    let inventory = player_inventory.single();
    return_pos.position = camera_transform.translation;

    camera_transform.translation.x = FISHING_ROOM_X;
    camera_transform.translation.y = FISHING_ROOM_Y;

    for cosmetic in inventory.cosmetics.iter() {
        println!("cosmetics: {}", cosmetic.name);
        if cosmetic.name == "Polarized Sun Glasses" {
            for mut vizi in fishes.iter_mut() {
                *vizi = Visibility::Visible;
                println!("Showing ZE FISH");
            }
        } else {
            for mut vizi in fishes.iter_mut() {
                *vizi = Visibility::Visible;
                println!("Showing ZE FISH");
            }
        }
    }
    //FISHING_ROOM_Y-308
    //spawn in powerbar
    //commands.spawn
    // power_bar_transform.translation.y = POWER_BAR_Y_OFFSET;
    // power_bar_info.power = 0.;

    //rd
    // rod_info.rotation = 0.;
    // rod_transform.rotation = Quat::from_rotation_z(rod_info.rotation);

    //new movmemnt system, rotation then space hold.
    //powerbar is space A, D are rotational
}

fn overworld_transition(
    mut camera: Query<&mut Transform, With<Camera>>,
    //mut power_bar: Query<(&mut Transform, &mut Power), With<Bar>>,
    return_pos: ResMut<PlayerReturnPos>,
) {
    let mut ct = camera.single_mut();
    //let (mut pb, mut power) = power_bar.single_mut();
    ct.translation = return_pos.position;

    //pb.translation.y = (POWER_BAR_Y_OFFSET);
    //power.meter = 0;
    //set powerbar back to 0
    //set rotation back to 0
}

fn power_bar_cast(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<FishingState>>,
    mut power_bar: Query<(&mut PowerBar, &mut Transform), With<PowerBar>>,
) {
    let (mut power_bar_info, mut power_bar_transform) = power_bar.single_mut();

    if input.pressed(TUG) {
        // Increase power
        power_bar_info.power = power_bar_info.power + POWER_FILL_SPEED * time.delta_seconds();

        if power_bar_info.power >= MAX_POWER {
            // Max power reached, release
            power_bar_info.power = MAX_POWER;
            next_state.set(FishingState::Casting);
        }

        power_bar_transform.translation.y = POWER_BAR_Y_OFFSET + power_bar_info.power;
    } else if input.just_released(TUG) {
        // Manual release
        next_state.set(FishingState::Casting);
    } else {
        return;
    }
}

fn switch_rod(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rod: Query<(&mut FishingRod, &mut Handle<Image>, &Transform), With<FishingRod>>,
    segments: Query<(Entity, &Mesh2dHandle), With<FishingRodSegment>>,
    mut player_inventory: Query<&mut PlayerInventory>,
) {
    if !input.just_pressed(SWITCH_ROD) {
        return;
    }

    let (mut rod_info, mut rod_texture, rod_transform) = rod.single_mut();
    let mut inventory = player_inventory.single_mut();

    inventory.rod_index = if inventory.rod_index == inventory.rods.len() - 1 {
        0
    } else {
        inventory.rod_index + 1
    };
    let current_rod = inventory.rods[inventory.rod_index].name;
    let new_type = RODS.get(current_rod).unwrap();

    rod_info.rod_type = new_type;
    materials.remove(&rod_info.material);
    rod_info.material = materials.add(new_type.blank_color);
    *rod_texture = asset_server.load(new_type.texture);
    rod_info.tip_pos = (rod_transform.translation.xy()
        + new_type.length * PIXELS_PER_METER * Vec2::from_angle(rod_info.rotation))
    .extend(0.);

    // Remove old segments
    for (segment_id, mesh_handle) in segments.iter() {
        meshes.remove(mesh_handle.id());
        commands.entity(segment_id).despawn();
    }

    // Create new segments
    let new_segment_count: usize = (new_type.length / BENDING_RESOLUTION) as usize;
    rod_info.segments = Vec::with_capacity(new_segment_count);

    for i in (0..new_segment_count).rev() {
        let l = i as f32 * BENDING_RESOLUTION;
        let radius = new_type.thickness * l / new_type.length;
        let radius_pixels = (radius * 750.).max(1.);

        let entity = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(radius_pixels, radius_pixels))),
                    material: rod_info.material.clone(),
                    ..default()
                },
                FishingRodSegment,
            ))
            .id();

        rod_info.segments.push(entity);
    }
}

fn switch_line(
    input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut line: Query<&mut FishingLine>,
    mut segments: Query<&Handle<ColorMaterial>, With<FishingLineSegment>>,
    mut player_inventory: Query<&mut PlayerInventory>,
) {
    if !input.just_pressed(SWITCH_LINE) {
        return;
    }

    let mut inventory = player_inventory.single_mut();
    let mut line_info = line.single_mut();

    inventory.line_index = if inventory.line_index == inventory.lines.len() - 1 {
        0
    } else {
        inventory.line_index + 1
    };
    let current_line = inventory.lines[inventory.line_index].name;
    line_info.line_type = LINES.get(current_line).unwrap();

    for material in segments.iter_mut() {
        materials.get_mut(material).unwrap().color = line_info.line_type.color;
    }
}

fn switch_lure(
    input: Res<ButtonInput<KeyCode>>,
    mut screen_lure: Query<&mut TextureAtlas, With<LureHUD>>,
    mut lure: Query<
        (&mut Lure, &mut PhysicsObject, &mut TextureAtlas),
        (With<Lure>, Without<LureHUD>),
    >,
    mut player_inventory: Query<&mut PlayerInventory>,
) {
    if !input.just_pressed(SWITCH_LURE) {
        return;
    }

    let mut inventory = player_inventory.single_mut();
    let mut screen_texture = screen_lure.single_mut();
    let (mut lure, mut physics, mut lure_texture) = lure.single_mut();

    inventory.lure_index = if inventory.lure_index == inventory.lures.len() - 1 {
        0
    } else {
        inventory.lure_index + 1
    };
    let current_lure = inventory.lures[inventory.lure_index].name;
    let new_lure = LURES.get(current_lure).unwrap();

    *lure = **new_lure;

    *physics = PhysicsObject {
        mass: new_lure.mass,
        position: Vec3::ZERO,
        rotation: Vec3::ZERO,
        velocity: Vec3::ZERO,
        forces: Forces::default(),
        cd: new_lure.cd,
        sa: new_lure.sa,
        waves: physics.waves,
    };

    screen_texture.index = new_lure.texture_index;
    lure_texture.index = new_lure.texture_index;
}

fn begin_cast(
    mut commands: Commands,
    power_bar: Query<&PowerBar>,
    mut line: Query<&mut FishingLine>,
    mut lure: Query<(Entity, &Lure, &mut Visibility, &mut PhysicsObject), With<Lure>>,
) {
    let power_bar_info = power_bar.single();
    let mut line_info = line.single_mut();
    let (entity_id, lure_info, mut lure_visibililty, mut lure_physics) = lure.single_mut();

    line_info.cast_distance = power_bar_info.power / MAX_POWER * MAX_CAST_DISTANCE;
    lure_physics.mass = lure_info.mass;
    lure_physics.cd = lure_info.cd;
    *lure_visibililty = Visibility::Visible;
    commands.entity(entity_id).insert(Hooked);
}

fn handle_debris(
    mut debris_details: Query<(&mut Transform, &DebrisType, &mut DebrisHooked), With<DebrisHooked>>,
    mut hooked_object: Query<(&Transform, &mut PhysicsObject), (With<Hooked>, Without<DebrisHooked>)>,
) {
    // Return if fish just caught
    if hooked_object.is_empty() {
        return;
    }

    let (hooked_object_transform, mut hooked_object_physics) = hooked_object.single_mut();

    for (mut debris_position, debris_info, mut debris_is_hooked) in debris_details.iter_mut() {
        let debris_pos = debris_position.translation;
        let attached_to = hooked_object_transform.translation;

        if debris_pos.y >= attached_to.y + debris_info.height / 2.
            || debris_pos.y <= attached_to.y - debris_info.height / 2.
            || debris_pos.x <= attached_to.x - debris_info.width / 2.
            || debris_pos.x >= attached_to.x + debris_info.width / 2.
        {
            if debris_is_hooked.hooked {
                debris_position.translation = attached_to;
            }
        } else {
            if !debris_is_hooked.hooked {
                debris_is_hooked.hooked = true;
                hooked_object_physics.mass += debris_info.mass;
                hooked_object_physics.cd = (
                    hooked_object_physics.cd.0 + debris_info.drag_increase,
                    hooked_object_physics.cd.1 + debris_info.drag_increase,
                );
            }

            debris_position.translation = attached_to;
        }
    }
}

fn is_done_reeling(
    mut commands: Commands,
    mut next_state: ResMut<NextState<FishingState>>,
    rod: Query<&FishingRod, With<FishingRod>>,
    mut cast_lure: Query<(Entity, &PhysicsObject), With<Hooked>>,
    debris: Query<(Entity, &DebrisHooked)>,
) {
    let rod_info = rod.single();
    let (entity_id, lure_physics) = cast_lure.single_mut();

    let distance = (lure_physics.position - rod_info.tip_pos).length();

    if distance > CATCH_MARGIN {
        return;
    }

    commands.entity(entity_id).remove::<Hooked>();

    // Despawn hooked debris
    for (entity_id, is_hooked) in debris.iter() {
        if is_hooked.hooked {
            commands.entity(entity_id).despawn();
        }
    }

    next_state.set(FishingState::Idle);
}

fn is_fish_caught(
    mut commands: Commands,
    mut player_inventory: Query<&mut PlayerInventory>,
    mut next_state: ResMut<NextState<FishingState>>,
    rod: Query<&FishingRod, With<FishingRod>>,
    mut hooked_object: Query<(Entity, &mut Fish, &mut PhysicsObject), With<Hooked>>,
    debris: Query<(Entity, &DebrisType, &DebrisHooked), Without<Hooked>>,
    lure: Query<&Lure>,
) {
    let rod_info = rod.single();
    let (entity_id, mut fish_details, mut fish_physics) = hooked_object.single_mut();
    let lure_info = lure.single();
    let mut inventory_info = player_inventory.single_mut();

    let distance = (fish_physics.position - rod_info.tip_pos).length();

    if distance < CATCH_MARGIN {
        fish_physics.mass -= lure_info.mass;

        // Remove weight of lure and debris from fish
        for (debris_id, debris_info, debris_hooked) in debris.iter() {
            if debris_hooked.hooked {
                fish_physics.mass -= debris_info.mass;
                commands.entity(debris_id).despawn();
            }
        }

        fish_details.is_caught = true;
        inventory_info.coins += fish_details.weight as u32 * 2;

        // Despawn fish
        commands.entity(entity_id).despawn();
        commands.entity(fish_physics.waves).despawn();
        fish_details.hooked_fish();

        next_state.set(FishingState::Idle);
    }
}

fn rod_rotate(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut fishing_rod: Query<(&mut FishingRod, &mut Transform), With<FishingRod>>,
) {
    let mut direction = 0.;

    if input.pressed(ROTATE_ROD_COUNTERLCOCKWISE) {
        direction += 1.;
    }

    if input.pressed(ROTATE_ROD_CLOCKWISE) {
        direction += -1.;
    }

    let (mut rod_info, mut rod_transform) = fishing_rod.single_mut();
    let new_rotation = rod_info.rotation + direction * ROD_ROTATION_SPEED * time.delta_seconds();
    rod_info.rotation = new_rotation.clamp(ROD_MIN_ROTATION, ROD_MAX_ROTATION);
    rod_transform.rotation = Quat::from_rotation_z(rod_info.rotation);
}

fn cast_line(
    time: Res<Time>,
    mut next_state: ResMut<NextState<FishingState>>,
    rod: Query<&FishingRod, With<FishingRod>>,
    mut line: Query<&mut FishingLine, With<FishingLine>>,
    mut lure: Query<(&mut Transform, &mut PhysicsObject), (With<Lure>, Without<FishingRod>)>,
    mut splash: Query<(&mut Splash, &mut Visibility), With<Splash>>,
) {
    let rod_info = rod.single();
    let mut line_info = line.single_mut();
    let (mut lure_transform, mut lure_physics) = lure.single_mut();
    let (mut splash_info, mut splash_visibility) = splash.single_mut();
    let angle_vector = Vec2::from_angle(rod_info.rotation).extend(0.);

    line_info.length =
        (line_info.length + CASTING_SPEED * time.delta_seconds()).min(line_info.cast_distance);
    line_info.end = rod_info.tip_pos + line_info.length * angle_vector;

    if line_info.length == line_info.cast_distance {
        // Cast finished
        lure_physics.forces.gravity = Vec3::new(0., 0., -GRAVITY * lure_physics.mass);
        line_info.length = line_info.cast_distance;
        splash_info.position = line_info.end.with_z(902.);
        *splash_visibility = Visibility::Visible;
        next_state.set(FishingState::ReelingUnhooked);
    }

    //setting the position of the lure along with the physics location of the lure.
    //also make sure that we are setting the lure to be a hooked object
    lure_physics.position = Vec3::new(line_info.end.x, line_info.end.y, lure_physics.position.z);
    lure_physics.forces.water = Vec3::ZERO;
    lure_transform.translation = line_info.end.with_z(950.);
}

fn animate_fishing_line(
    rod: Query<&FishingRod, With<FishingRod>>,
    hooked_fish: Query<(&Species, &PhysicsObject), (With<Fish>, With<Hooked>)>,
    mut line: Query<&mut FishingLine, With<FishingLine>>,
    mut lure: Query<&PhysicsObject, With<Lure>>,
    state: Res<State<FishingState>>,
    time: Res<Time>,
) {
    let rod_info = rod.single();
    let mut line_info = line.single_mut();

    line_info.start = rod_info.tip_pos;

    if !hooked_fish.is_empty() {
        // Reeling hooked
        let (fish_species, fish_physics) = hooked_fish.single();
        let fish_offset = fish_species
            .hook_pos
            .rotate(Vec2::from_angle(fish_physics.rotation.z));
        let fish_pos = fish_physics.position + fish_offset.extend(0.);
        line_info.end = fish_pos;
    } else if state.eq(&FishingState::ReelingUnhooked) {
        // Reeling unhooked
        let lure_physics = lure.single_mut();
        line_info.end = lure_physics.position;
    } else {
        // Idle
        line_info.end = line_info.start;
    }
}

fn reset_interface(
    mut power_bar: Query<&mut PowerBar>,
    mut line: Query<&mut FishingLine, With<FishingLine>>,
    mut splash: Query<&mut TextureAtlas, With<Splash>>,
    mut lure: Query<(&mut PhysicsObject, &mut Visibility), With<Lure>>,
) {
    let mut power_bar_info = power_bar.single_mut();
    let mut line_info = line.single_mut();
    let mut splash_texture = splash.single_mut();
    let (mut lure_physics, mut lure_visibility) = lure.single_mut();

    line_info.length = 0.;
    line_info.start = Vec3::ZERO;
    line_info.end = Vec3::ZERO;
    lure_physics.position.z = 0.;
    lure_physics.velocity = Vec3::ZERO;
    lure_physics.forces = Forces::default();
    *lure_visibility = Visibility::Hidden;
    splash_texture.index = 0;
    power_bar_info.power = 0.;
}

pub fn move_physics_objects(
    mut objects: Query<(&PhysicsObject, &mut Sprite, &mut Transform), With<PhysicsObject>>,
) {
    for (physics_object, mut sprite, mut transform) in objects.iter_mut() {
        transform.translation = physics_object.position.with_z(transform.translation.z);
        transform.rotation = Quat::from_rotation_z(physics_object.rotation.z);

        let new_alpha = if physics_object.position.z > 0. {
            1.
        } else {
            1. / (1. - physics_object.position.z / DEPTH_DECAY).powi(2)
        };

        sprite.color.set_alpha(new_alpha);
    }
}

fn adjust_fishing_line_size(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut line: Query<&mut FishingLine>,
    mut line_segments: Query<
        (&mut Visibility, &mut Transform, &Handle<ColorMaterial>),
        With<FishingLineSegment>,
    >,
) {
    let mut line_info = line.single_mut();

    let pos_delta = line_info.end - line_info.start;
    let segment_count = pos_delta.with_z(0.).length() as usize;
    let segments = line_info.segments.len();

    if segments < segment_count {
        for _i in segments..segment_count {
            let entity_id = commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(
                            meshes.add(Rectangle::new(FishingLine::WIDTH, FishingLine::WIDTH)),
                        ),
                        material: materials
                            .add(ColorMaterial::from_color(line_info.line_type.color)),
                        ..default()
                    },
                    FishingLineSegment,
                ))
                .id();

            line_info.segments.push(entity_id);
        }
    } else if segments > segment_count {
        for i in segment_count..segments {
            let entity_id = line_info.segments[i];
            let (mut visibility, mut _transform, _material) =
                line_segments.get_mut(entity_id).unwrap();

            *visibility = Visibility::Hidden;
        }
    }
}

fn draw_fishing_line(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut line: Query<&mut FishingLine>,
    mut line_segments: Query<
        (&mut Visibility, &mut Transform, &Handle<ColorMaterial>),
        With<FishingLineSegment>,
    >,
) {
    if line_segments.is_empty() {
        return;
    }

    let line_info = line.single_mut();

    let pos_delta = line_info.end - line_info.start;
    let segment_count = pos_delta.with_z(0.).length() as usize;

    for i in 0..segment_count {
        let entity_id = line_info.segments[i];
        let (mut visibility, mut transform, material_handle) =
            line_segments.get_mut(entity_id).unwrap();
        let position = line_info.start + i as f32 / segment_count as f32 * pos_delta;

        let new_alpha = if position.z > 0. {
            1.
        } else {
            1. / (1. - position.z / DEPTH_DECAY).powi(2)
        };

        *visibility = Visibility::Visible;
        transform.translation = position.with_z(950.);
        let material = materials.get_mut(material_handle).unwrap();
        material.color = material.color.with_alpha(new_alpha);
    }
}

fn animate_splash(
    mut splash: Query<
        (
            &Splash,
            &mut Transform,
            &mut Visibility,
            &mut TextureAtlas,
            &mut AnimationTimer,
        ),
        With<Splash>,
    >,
    time: Res<Time>,
) {
    let (splash, mut transform, mut visibility, mut texture, mut timer) = splash.single_mut();

    if *visibility == Visibility::Hidden {
        return;
    }

    transform.translation = splash.position;

    // Animate splash
    if texture.index < 3 {
        timer.tick(time.delta());

        if timer.just_finished() {
            if texture.index == 2 {
                *visibility = Visibility::Hidden;
            } else {
                texture.index += 1;
            }
        }
    }
}

fn animate_waves(
    objects: Query<(&Visibility, &PhysicsObject), With<PhysicsObject>>,
    mut waves: Query<
        (&mut TextureAtlas, &mut Transform, &mut Visibility),
        (With<Wave>, Without<PhysicsObject>),
    >,
) {
    for (object_visibility, physics_object) in objects.iter() {
        let (mut wave_texture, mut wave_transform, mut wave_visibility) =
            waves.get_mut(physics_object.waves).unwrap();

        if object_visibility == Visibility::Hidden || physics_object.position.z > 0. {
            *wave_visibility = Visibility::Hidden;
            continue;
        }

        let decay_factor = (1. - physics_object.position.z / DEPTH_DECAY).powi(3);
        let magnitude = physics_object.forces.water.length() / decay_factor;

        if magnitude < 50. {
            *wave_visibility = Visibility::Hidden;
            continue;
        } else if magnitude < 200. {
            wave_texture.index = 0;
        } else if magnitude < 400. {
            wave_texture.index = 1;
        } else if magnitude < 600. {
            wave_texture.index = 2;
        } else {
            wave_texture.index = 3;
        }

        *wave_visibility = Visibility::Visible;

        wave_transform.translation = physics_object.position.with_z(902.);
        wave_transform.rotation = Quat::from_rotation_z(
            f32::atan2(physics_object.forces.water.y, physics_object.forces.water.x) - PI / 2.,
        );
    }
}
