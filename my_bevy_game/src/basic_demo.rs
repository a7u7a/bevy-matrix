// Explicit imports - only what we need for this demo
use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::prelude::Resource;
use bevy::time::{Time, Timer, TimerMode};

/// A minimal plugin demonstrating basic Bevy ECS features
/// This should work on both windowed (Mac) and headless (Pi) platforms
pub struct BasicDemoPlugin;

impl Plugin for BasicDemoPlugin {
    fn build(&self, app: &mut App) {
        // Insert our timer resource
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));

        // Add startup system to create entities
        app.add_systems(Startup, add_people);

        // Add update systems that run every frame
        app.add_systems(Update, (update_people, greet_people));
    }
}

// Components
#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

// Resource
#[derive(Resource)]
struct GreetTimer(Timer);

// Startup system - runs once at app start
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

// Update system - runs every frame
fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break;
        }
    }
}

// Update system - runs every frame, prints on timer
fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // Tick the timer with the time elapsed since last frame
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}
