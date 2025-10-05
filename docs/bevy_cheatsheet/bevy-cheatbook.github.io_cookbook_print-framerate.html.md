---
url: "https://bevy-cheatbook.github.io/cookbook/print-framerate.html"
title: "Show Framerate - Unofficial Bevy Cheat Book"
---

- Auto
- Light
- Rust
- Coal
- Navy
- Ayu

# Unofficial Bevy Cheat Book

[Print this book](https://bevy-cheatbook.github.io/print.html "Print this book")[Suggest an edit](https://github.com/bevy-cheatbook/bevy-cheatbook/issues/new "Suggest an edit")

| [Bevy Version:](https://bevy-cheatbook.github.io/introduction.html#maintenance-policy) | [0.13](https://bevyengine.org/news/bevy-0-13) | (outdated!) |
| --- | --- | --- |

As this page is outdated, please refer to Bevy's official migration guides while reading,
to cover the differences:
[0.13 to 0.14](https://bevyengine.org/learn/migration-guides/0-13-to-0-14/),
[0.14 to 0.15](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/),
[0.15 to 0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/).

I apologize for the inconvenience. I will update the page as soon as I find the time.

* * *

You can use Bevy's builtin diagnostics to measure framerate (FPS), for
monitoring performance.

To enable it, add Bevy's diagnostic plugin to your [app](https://bevy-cheatbook.github.io/programming/app-builder.html):

```rust no_run noplayground hljs

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
app.add_plugins(FrameTimeDiagnosticsPlugin::default());
```

The simplest way to use it is to print the diagnostics to the console
( [log](https://bevy-cheatbook.github.io/fundamentals/log.html)). If you want to only do it in dev builds, you can add
a conditional-compilation attribute.

```rust no_run noplayground hljs

#[cfg(debug_assertions)] // debug/dev builds only
{
    use bevy::diagnostic::LogDiagnosticsPlugin;
    app.add_plugins(LogDiagnosticsPlugin::default());
}
```

UPDATE! I have now released a Bevy plugin which provides a much better
version of the code on this page, ready for you to use! Consider trying
my [`iyes_perf_ui`](https://github.com/IyesGames/iyes_perf_ui) plugin!

Bevy maintainers have expressed interest in upstreaming it, and we will
try to make it official in the next Bevy release (0.14)!

For now, I am also keeping the old code example below in the book, for
completeness:

* * *

You can use Bevy UI to create an in-game FPS counter.

It is recommended that you create a new UI root (entity without
a parent) with absolute positioning, so that you can control the
exact position where the FPS counter appears, and so it doesn't
affect the rest of your UI.

Here is some example code showing you how to make a very nice-looking and
readable FPS counter:

`Code Example (Long):`

```rust no_run noplayground hljs

use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

fn setup_fps_counter(
    mut commands: Commands,
) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands.spawn((
        FpsRoot,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                // position it at the top-right corner
                // 1% away from the top window edge
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                bottom: Val::Auto,
                left: Val::Auto,
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    // create our text
    let text_fps = commands.spawn((
        FpsText,
        TextBundle {
            // use two sections, so it is easy to update just the number
            text: Text::from_sections([\
                TextSection {\
                    value: "FPS: ".into(),\
                    style: TextStyle {\
                        font_size: 16.0,\
                        color: Color::WHITE,\
                        // if you want to use your game's font asset,\
                        // uncomment this and provide the handle:\
                        // font: my_font_handle\
                        ..default()\
                    }\
                },\
                TextSection {\
                    value: " N/A".into(),\
                    style: TextStyle {\
                        font_size: 16.0,\
                        color: Color::WHITE,\
                        // if you want to use your game's font asset,\
                        // uncomment this and provide the handle:\
                        // font: my_font_handle\
                        ..default()\
                    }\
                },\
            ]),
            ..Default::default()
        },
    )).id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb(
                    (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                    1.0,
                    0.0,
                )
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(
                    1.0,
                    ((value - 30.0) / (60.0 - 30.0)) as f32,
                    0.0,
                )
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
```

```rust no_run noplayground hljs

app.add_systems(Startup, setup_fps_counter);
app.add_systems(Update, (
    fps_text_update_system,
    fps_counter_showhide,
));
```

[GitHub Sponsors](https://github.com/sponsors/inodentry) [Patreon](https://patreon.com/iyesgames) [Bitcoin](bitcoin:bc1qaf32uqsg6mngw9g4aqc3l2jvuv46qx0zw2438p) If you like this book, please donate to support me!

I also offer professional tutoring / private lessons for Bevy and Rust. [Contact me](https://bevy-cheatbook.github.io/contact.html) if interested!