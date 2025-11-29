use bevy::prelude::*;

mod display;
use display::{Backend, DisplayBackend, DisplayResource};

fn main() {
    let backend = Backend::new().expect("Failed to initialize display");
    let display_resource = DisplayResource(Box::new(backend));
    
    let mut app = App::new();
    
    #[cfg(feature = "window")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy Game".into(),
            resolution: (64, 64).into(),
            ..default()
        }),
        ..default()
    }));
    
    #[cfg(not(feature = "window"))]
    app.add_plugins(MinimalPlugins);
    
    app.insert_resource(display_resource)
        .add_systems(Startup, setup)
        .add_systems(Update, (render_frame, hello_world, (update_people, greet_people).chain()))
        .run();
}

fn setup(mut commands: Commands) {
    #[cfg(feature = "window")]
    commands.spawn(Camera2d);
    
    // Keep the original people spawning
    add_people(commands);
}

fn render_frame(
    mut backend: ResMut<DisplayResource>,
) {
    // Extract frame pixels and write to backend
    // For now, create a simple test pattern - alternating red and blue pixels
    let mut pixels = vec![0u8; 64 * 64 * 3];
    for y in 0..64 {
        for x in 0..64 {
            let idx = ((y * 64 + x) * 3) as usize;
            // Create a simple test pattern
            if (x + y) % 2 == 0 {
                pixels[idx] = 255;     // Red
                pixels[idx + 1] = 0;   // Green
                pixels[idx + 2] = 0;   // Blue
            } else {
                pixels[idx] = 0;       // Red
                pixels[idx + 1] = 0;   // Green
                pixels[idx + 2] = 255; // Blue
            }
        }
    }
    backend.0.write_frame(&pixels).ok();
}

// Keeping for reference
fn hello_world() {
    println!("hello world!");
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

// Iterate over every Name component for entities that also have a Person component
fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
}

// Your First mutable Query
fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}
