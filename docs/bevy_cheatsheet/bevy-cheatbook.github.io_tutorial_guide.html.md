---
url: "https://bevy-cheatbook.github.io/tutorial/guide.html"
title: "Guided Tour - Unofficial Bevy Cheat Book"
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

Welcome to Bevy! :) We are glad to have you in our community!

This page will guide you through this book, to help you gain comprehensive
knowledge of how to work with Bevy. The topics are structured in an order
that makes sense for learning: from basics to advanced.

It is just a suggestion to help you navigate. Feel free to jump around the book
and read whatever interests you. The main table-of-contents (the left sidebar)
was designed to be a reference for Bevy users of any skill level.

* * *

Make sure to also look at [the official Bevy examples](https://github.com/bevyengine/bevy/tree/latest/examples#examples). If
you need help, use [GitHub Discussions](https://github.com/bevyengine/bevy/discussions), or feel welcome
to join us to chat and ask for help in [Discord](https://discord.gg/bevy).

If you run into issues, be sure to check the
[Common Pitfalls](https://bevy-cheatbook.github.io/pitfalls.html) chapter, to see if this book has something
to help you. Solutions to some of the most common issues that Bevy community
members have encountered are documented there.

These are the absolute essentials of using Bevy. Every Bevy project, even a
simple one, would require you to be familiar with these concepts.

You could conceivably make something like a simple game-jam game or prototype,
using just this knowledge. Though, as your project grows, you will likely
quickly need to learn more.

- [Bevy Setup Tips](https://bevy-cheatbook.github.io/setup.html)
  - [Getting Started](https://bevy-cheatbook.github.io/setup/getting-started.html)
- [Bevy Programming Framework](https://bevy-cheatbook.github.io/programming.html)
  - [Intro to ECS](https://bevy-cheatbook.github.io/programming/ecs-intro.html)
  - [Entities, Components](https://bevy-cheatbook.github.io/programming/intro-data.html#entities--components)
  - [Bundles](https://bevy-cheatbook.github.io/programming/bundle.html)
  - [Resources](https://bevy-cheatbook.github.io/programming/res.html)
  - [Systems](https://bevy-cheatbook.github.io/programming/systems.html)
  - [App Builder](https://bevy-cheatbook.github.io/programming/app-builder.html)
  - [Queries](https://bevy-cheatbook.github.io/programming/queries.html)
  - [Commands](https://bevy-cheatbook.github.io/programming/commands.html)
- [Game Engine Fundamentals](https://bevy-cheatbook.github.io/fundamentals.html)
  - [Coordinate System](https://bevy-cheatbook.github.io/fundamentals/coords.html)
  - [Transforms](https://bevy-cheatbook.github.io/fundamentals/transforms.html)
  - [Time and Timers](https://bevy-cheatbook.github.io/fundamentals/time.html)
- [General Graphics Features](https://bevy-cheatbook.github.io/graphics.html)
  - [Cameras](https://bevy-cheatbook.github.io/graphics/camera.html)
- [Bevy Asset Management](https://bevy-cheatbook.github.io/assets.html)
  - [Load Assets with AssetServer](https://bevy-cheatbook.github.io/assets/assetserver.html)
  - [Handles](https://bevy-cheatbook.github.io/assets/handles.html)
- [Input Handling](https://bevy-cheatbook.github.io/input.html)
  - [Keyboard](https://bevy-cheatbook.github.io/input/keyboard.html)
  - [Mouse](https://bevy-cheatbook.github.io/input/mouse.html)
  - [Gamepad (Controller)](https://bevy-cheatbook.github.io/input/gamepad.html)
  - [Touchscreen](https://bevy-cheatbook.github.io/input/touch.html)
- [Window Management](https://bevy-cheatbook.github.io/window.html)
  - [Window Properties](https://bevy-cheatbook.github.io/window/props.html)
  - [Change the Background Color](https://bevy-cheatbook.github.io/window/clear-color.html)
- [Audio](https://bevy-cheatbook.github.io/audio.html)
  - [Playing Sounds](https://bevy-cheatbook.github.io/audio/basic.html)

You will likely need to learn most of these topics to make a non-trivial Bevy
project. After you are confident with the basics, you should learn these.

- [Bevy Programming Framework](https://bevy-cheatbook.github.io/programming.html)
  - [Events](https://bevy-cheatbook.github.io/programming/events.html)
  - [System Order of Execution](https://bevy-cheatbook.github.io/programming/system-order.html)
  - [Run Conditions](https://bevy-cheatbook.github.io/programming/run-criteria.html)
  - [System Sets](https://bevy-cheatbook.github.io/programming/system-sets.html)
  - [Local Resources](https://bevy-cheatbook.github.io/programming/local.html)
  - [Schedules](https://bevy-cheatbook.github.io/programming/schedules.html)
  - [States](https://bevy-cheatbook.github.io/programming/states.html)
  - [Plugins](https://bevy-cheatbook.github.io/programming/plugins.html)
  - [Change Detection](https://bevy-cheatbook.github.io/programming/change-detection.html)
- [Game Engine Fundamentals](https://bevy-cheatbook.github.io/fundamentals.html)
  - [Parent/Child Hierarchies](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html)
  - [Visibility](https://bevy-cheatbook.github.io/fundamentals/visibility.html)
  - [Logging / Console Messages](https://bevy-cheatbook.github.io/fundamentals/log.html)
- [Input Handling](https://bevy-cheatbook.github.io/input.html)
  - [Convert cursor to world coordinates](https://bevy-cheatbook.github.io/cookbook/cursor2world.html)
- [Bevy Asset Management](https://bevy-cheatbook.github.io/assets.html)
  - [Access the Asset Data](https://bevy-cheatbook.github.io/assets/data.html)
  - [Hot-Reloading Assets](https://bevy-cheatbook.github.io/assets/hot-reload.html)
- [Bevy Setup Tips](https://bevy-cheatbook.github.io/setup.html)
  - [Bevy Dev Tools and Editors](https://bevy-cheatbook.github.io/setup/bevy-tools.html)
  - [Community Plugin Ecosystem](https://bevy-cheatbook.github.io/setup/unofficial-plugins.html)
- [Audio](https://bevy-cheatbook.github.io/audio.html):

  - [Spatial Audio](https://bevy-cheatbook.github.io/audio/spatial.html)

These are more specialized topics. You may need some of them, depending on your
project.

- [Bevy Programming Framework](https://bevy-cheatbook.github.io/programming.html)
  - [Direct World Access](https://bevy-cheatbook.github.io/programming/world.html)
  - [Exclusive Systems](https://bevy-cheatbook.github.io/programming/exclusive.html)
  - [Param Sets](https://bevy-cheatbook.github.io/programming/paramset.html)
  - [System Piping](https://bevy-cheatbook.github.io/programming/system-piping.html)
- [Game Engine Fundamentals](https://bevy-cheatbook.github.io/fundamentals.html)
  - [Fixed Timestep](https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html)
- [General Graphics Features](https://bevy-cheatbook.github.io/graphics.html)
  - [HDR, Tonemapping](https://bevy-cheatbook.github.io/graphics/hdr-tonemap.html)
  - [Bloom](https://bevy-cheatbook.github.io/graphics/bloom.html)
- [Bevy Asset Management](https://bevy-cheatbook.github.io/assets.html)
  - [React to Changes with Asset Events](https://bevy-cheatbook.github.io/assets/assetevent.html)
  - [Track asset loading progress](https://bevy-cheatbook.github.io/assets/ready.html)
- [Programming Patterns](https://bevy-cheatbook.github.io/patterns.html)
  - [Write tests for systems](https://bevy-cheatbook.github.io/patterns/system-tests.html)
  - [Generic Systems](https://bevy-cheatbook.github.io/patterns/generic-systems.html)
  - [Manual Event Clearing](https://bevy-cheatbook.github.io/patterns/manual-event-clear.html)
- [Window Management](https://bevy-cheatbook.github.io/window.html)
  - [Grab/Capture the Mouse Cursor](https://bevy-cheatbook.github.io/window/mouse-grab.html)
  - [Set the Window Icon](https://bevy-cheatbook.github.io/window/icon.html)
- [Audio](https://bevy-cheatbook.github.io/audio.html)
  - [Custom Audio Streams](https://bevy-cheatbook.github.io/audio/custom.html)

These topics are for niche technical situations. You can learn them, if you want
to know more about how Bevy works internally, extend the engine with custom
functionality, or do other advanced things with Bevy.

- [Bevy Programming Framework](https://bevy-cheatbook.github.io/programming.html)
  - [Non-Send](https://bevy-cheatbook.github.io/programming/non-send.html)
- [Programming Patterns](https://bevy-cheatbook.github.io/patterns.html)
  - [Component Storage](https://bevy-cheatbook.github.io/patterns/component-storage.html)
- [Input Handling](https://bevy-cheatbook.github.io/input.html)
  - [Drag-and-Drop files](https://bevy-cheatbook.github.io/input/dnd.html)
  - [IME for advanced text input](https://bevy-cheatbook.github.io/input/ime.html)
- [Bevy Setup Tips](https://bevy-cheatbook.github.io/setup.html)
  - [Customizing Bevy (cargo crates and features)](https://bevy-cheatbook.github.io/setup/bevy-config.html)
  - [Using bleeding-edge Bevy (main)](https://bevy-cheatbook.github.io/setup/bevy-git.html)
- [Bevy Render (GPU) Framework](https://bevy-cheatbook.github.io/gpu.html)
  - [Render Architecture Overview](https://bevy-cheatbook.github.io/gpu/intro.html)
  - [Render Sets](https://bevy-cheatbook.github.io/gpu/stages.html)

[GitHub Sponsors](https://github.com/sponsors/inodentry) [Patreon](https://patreon.com/iyesgames) [Bitcoin](bitcoin:bc1qaf32uqsg6mngw9g4aqc3l2jvuv46qx0zw2438p) If you like this book, please donate to support me!

I also offer professional tutoring / private lessons for Bevy and Rust. [Contact me](https://bevy-cheatbook.github.io/contact.html) if interested!