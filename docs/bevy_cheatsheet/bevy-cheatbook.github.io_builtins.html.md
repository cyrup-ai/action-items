---
url: "https://bevy-cheatbook.github.io/builtins.html"
title: "List of Bevy Builtins - Unofficial Bevy Cheat Book"
---

- Auto
- Light
- Rust
- Coal
- Navy
- Ayu

# Unofficial Bevy Cheat Book

[Print this book](https://bevy-cheatbook.github.io/print.html "Print this book")[Suggest an edit](https://github.com/bevy-cheatbook/bevy-cheatbook/issues/new "Suggest an edit")

| [Bevy Version:](https://bevy-cheatbook.github.io/introduction.html#maintenance-policy) | [0.11](https://bevyengine.org/news/bevy-0-11) | (outdated!) |
| --- | --- | --- |

As this page is outdated, please refer to Bevy's official migration guides while reading,
to cover the differences:
[0.11 to 0.12](https://bevyengine.org/learn/migration-guides/0-11-to-0-12/),
[0.12 to 0.13](https://bevyengine.org/learn/migration-guides/0-12-to-0-13/),
[0.13 to 0.14](https://bevyengine.org/learn/migration-guides/0-13-to-0-14/),
[0.14 to 0.15](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/),
[0.15 to 0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/).

I apologize for the inconvenience. I will update the page as soon as I find the time.

* * *

This page is a quick condensed listing of all the important things provided
by Bevy.

- [SystemParams](https://bevy-cheatbook.github.io/builtins.html#systemparams)
- [Assets](https://bevy-cheatbook.github.io/builtins.html#assets)
- [File Formats](https://bevy-cheatbook.github.io/builtins.html#file-formats)
- [GLTF Asset Labels](https://bevy-cheatbook.github.io/builtins.html#gltf-asset-labels)
- [Shader Imports](https://bevy-cheatbook.github.io/builtins.html#shader-imports)
- [`wgpu` Backends](https://bevy-cheatbook.github.io/builtins.html#wgpu-backends)
- [Schedules](https://bevy-cheatbook.github.io/builtins.html#schedules)
- [Run Conditions](https://bevy-cheatbook.github.io/builtins.html#run-conditions)
- [Plugins](https://bevy-cheatbook.github.io/builtins.html#plugins)
- [Bundles](https://bevy-cheatbook.github.io/builtins.html#bundles)
- [Resources (Configuration)](https://bevy-cheatbook.github.io/builtins.html#configuration-resources)
- [Resources (Engine User)](https://bevy-cheatbook.github.io/builtins.html#engine-resources)
  - [Main World](https://bevy-cheatbook.github.io/builtins.html#engine-resources)
  - [Render World](https://bevy-cheatbook.github.io/builtins.html#render-world-resources)
  - [Low-Level `wgpu` access](https://bevy-cheatbook.github.io/builtins.html#low-level-wgpu-resources)
- [Resources (Input)](https://bevy-cheatbook.github.io/builtins.html#input-handling-resources)
- [Events (Input)](https://bevy-cheatbook.github.io/builtins.html#input-events)
- [Events (Engine)](https://bevy-cheatbook.github.io/builtins.html#engine-events)
- [Events (System/Control)](https://bevy-cheatbook.github.io/builtins.html#system-and-control-events)
- [Components](https://bevy-cheatbook.github.io/builtins.html#components)

These are all the special types that can be used as [system](https://bevy-cheatbook.github.io/programming/systems.html) parameters.

[(List in API Docs)](https://docs.rs/bevy/0.11/bevy/ecs/system/trait.SystemParam.html#implementors)

In regular [systems](https://bevy-cheatbook.github.io/programming/systems.html):

- [`Commands`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.Commands.html):
Manipulate the ECS using [commands](https://bevy-cheatbook.github.io/programming/commands.html)
- [`Query<T, F = ()>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.Query.html) (can contain tuples of up to 15 types):
Access to [entities and components](https://bevy-cheatbook.github.io/programming/intro-data.html#entities--components)
- [`Res<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.Res.html):
Shared access to a [resource](https://bevy-cheatbook.github.io/programming/res.html)
- [`ResMut<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.ResMut.html):
Exclusive (mutable) access to a [resource](https://bevy-cheatbook.github.io/programming/res.html)
- `Option<Res<T>>`:
Shared access to a resource that may not exist
- `Option<ResMut<T>>`:
Exclusive (mutable) access to a resource that may not exist
- [`Local<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.Local.html):
Data [local](https://bevy-cheatbook.github.io/programming/local.html) to the system
- [`EventReader<T>`](https://docs.rs/bevy/0.11/bevy/ecs/event/struct.EventReader.html):
Receive [events](https://bevy-cheatbook.github.io/programming/events.html)
- [`EventWriter<T>`](https://docs.rs/bevy/0.11/bevy/ecs/event/struct.EventWriter.html):
Send [events](https://bevy-cheatbook.github.io/programming/events.html)
- [`&World`](https://docs.rs/bevy/0.11/bevy/ecs/world/struct.World.html):
Read-only [direct access to the ECS World](https://bevy-cheatbook.github.io/programming/world.html)
- [`ParamSet<...>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.ParamSet.html) (with up to 8 params):
Resolve [conflicts between incompatible system parameters](https://bevy-cheatbook.github.io/programming/paramset.html)
- [`Deferred<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.Deferred.html):
Custom ["deferred mutation"](https://bevy-cheatbook.github.io/programming/deferred.html), similar to `Commands`, but for your own things
- [`RemovedComponents<T>`](https://docs.rs/bevy/0.11/bevy/ecs/removal_detection/struct.RemovedComponents.html):
[Removal detection](https://bevy-cheatbook.github.io/programming/change-detection.html#removal-detection)
- [`Gizmos`](https://docs.rs/bevy/0.11/bevy/gizmos/gizmos/struct.Gizmos.html):
A way to [draw lines and shapes](https://bevy-cheatbook.github.io/fundamentals/gizmos.html) on the screen for debugging and dev purposes
- [`Diagnostics`](https://docs.rs/bevy/0.11/bevy/diagnostic/struct.Diagnostics.html):
A way to [report measurements/debug data](https://bevy-cheatbook.github.io/fundamentals/diagnostics.html) to Bevy for tracking and visualization
- [`SystemName`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.SystemName.html):
The name (string) of the system, may be useful for debugging
- [`ParallelCommands`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.ParallelCommands.html):
Abstraction to help use `Commands` when you will do your own parallelism
- [`WorldId`](https://docs.rs/bevy/0.11/bevy/ecs/world/struct.WorldId.html):
The World ID of the [world](https://bevy-cheatbook.github.io/programming/world.html) the system is running on
- [`ComponentIdFor<T>`](https://docs.rs/bevy/0.11/bevy/ecs/component/struct.ComponentIdFor.html):
Get the [`ComponentId`](https://docs.rs/bevy/0.11/bevy/ecs/component/struct.ComponentId.html) of a given [component](https://bevy-cheatbook.github.io/programming/ec.html#components) type
- [`Entities`](https://docs.rs/bevy/0.11/bevy/ecs/entity/struct.Entities.html):
Low-level ECS metadata: All entities
- [`Components`](https://docs.rs/bevy/0.11/bevy/ecs/component/struct.Components.html):
Low-level ECS metadata: All components
- [`Bundles`](https://docs.rs/bevy/0.11/bevy/ecs/bundle/struct.Bundles.html):
Low-level ECS metadata: All bundles
- [`Archetypes`](https://docs.rs/bevy/0.11/bevy/ecs/archetype/struct.Archetypes.html):
Low-level ECS metadata: All archetypes
- [`SystemChangeTick`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.SystemChangeTick.html):
Low-level ECS metadata: Tick used for change detection
- [`NonSend<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.NonSend.html):
Shared access to [Non- `Send`](https://bevy-cheatbook.github.io/programming/non-send.html) (main thread only) data
- [`NonSendMut<T>`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.NonSendMut.html):
Exclusive access to [Non- `Send`](https://bevy-cheatbook.github.io/programming/non-send.html) (main thread only) data
- `Option<NonSend<T>>`:
Shared access to [Non- `Send`](https://bevy-cheatbook.github.io/programming/non-send.html) (main thread only) data that may not exist
- `Option<NonSendMut<T>>`:
Exclusive access to [Non- `Send`](https://bevy-cheatbook.github.io/programming/non-send.html) (main thread only) data that may not exist
- [`StaticSystemParam`](https://docs.rs/bevy/0.11/bevy/ecs/system/struct.StaticSystemParam.html):
Helper for generic system abstractions, to avoid lifetime annotations
- tuples containing any of these types, with up to 16 members

In [exclusive systems](https://bevy-cheatbook.github.io/programming/exclusive.html):

- \[ `&mut World`\]:
Full [direct access to the ECS World](https://bevy-cheatbook.github.io/programming/world.html)
- \[ `Local<T>`\]:
Data [local](https://bevy-cheatbook.github.io/programming/local.html) to the system
- \[ `&mut SystemState<P>`\]\[ `SystemState`\]:
Emulates a regular system, allowing you to easily access data from the World.
`P` are the system parameters.
- \[ `&mut QueryState<Q, F = ()>`\]\[ `QueryState`\]:
Allows you to perform queries on the World, similar to a \[ `Query`\] in regular systems.

Your function can have a maximum of 16 total parameters. If you need more,
group them into tuples to work around the limit. Tuples can contain up to
16 members, but can be nested indefinitely.

Systems running during the [Extract schedule](https://bevy-cheatbook.github.io/TODO.html) can also use
[`Extract<T>`](https://docs.rs/bevy/0.11/bevy/render/struct.Extract.html), to access data from the Main World instead of the
Render World. `T` can be any read-only system parameter type.

[(more info about working with assets)](https://bevy-cheatbook.github.io/assets.html)

These are the Asset types registered by Bevy by default.

- [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html):
Pixel data, used as a texture for 2D and 3D rendering;
also contains the [`SamplerDescriptor`](https://docs.rs/bevy/0.11/bevy/render/render_resource/struct.SamplerDescriptor.html) for texture filtering settings
- [`TextureAtlas`](https://docs.rs/bevy/0.11/bevy/sprite/struct.TextureAtlas.html):
2D "Sprite Sheet" defining sub-images within a single larger image
- [`Mesh`](https://docs.rs/bevy/0.11/bevy/render/mesh/struct.Mesh.html):
3D Mesh (geometry data), contains vertex attributes (like position, UVs, normals)
- [`Shader`](https://docs.rs/bevy/0.11/bevy/render/render_resource/struct.Shader.html):
GPU shader code, in one of the supported languages (WGSL/SPIR-V/GLSL)
- [`ColorMaterial`](https://docs.rs/bevy/0.11/bevy/sprite/struct.ColorMaterial.html):
Basic "2D material": contains color, optionally an image
- [`StandardMaterial`](https://docs.rs/bevy/0.11/bevy/pbr/struct.StandardMaterial.html):
"3D material" with support for Physically-Based Rendering
- [`AnimationClip`](https://docs.rs/bevy/0.11/bevy/animation/struct.AnimationClip.html):
Data for a single animation sequence, can be used with [`AnimationPlayer`](https://docs.rs/bevy/0.11/bevy/animation/struct.AnimationPlayer.html)
- [`Font`](https://docs.rs/bevy/0.11/bevy/text/struct.Font.html):
Font data used for text rendering
- [`Scene`](https://docs.rs/bevy/0.11/bevy/scene/struct.Scene.html):
Scene composed of literal ECS entities to instantiate
- [`DynamicScene`](https://docs.rs/bevy/0.11/bevy/scene/struct.DynamicScene.html):
Scene composed with dynamic typing and reflection
- [`Gltf`](https://docs.rs/bevy/0.11/bevy/gltf/struct.Gltf.html):
[GLTF Master Asset](https://bevy-cheatbook.github.io/3d/gltf.html#gltf-master-asset): index of the entire contents of a GLTF file
- [`GltfNode`](https://docs.rs/bevy/0.11/bevy/gltf/struct.GltfNode.html):
Logical GLTF object in a scene
- [`GltfMesh`](https://docs.rs/bevy/0.11/bevy/gltf/struct.GltfMesh.html):
Logical GLTF 3D model, consisting of multiple `GltfPrimitive` s
- [`GltfPrimitive`](https://docs.rs/bevy/0.11/bevy/gltf/struct.GltfPrimitive.html):
Single unit to be rendered, contains the Mesh and Material to use
- [`AudioSource`](https://docs.rs/bevy/0.11/bevy/audio/struct.AudioSource.html):
Audio data for `bevy_audio`
- [`FontAtlasSet`](https://docs.rs/bevy/0.11/bevy/text/struct.FontAtlasSet.html):
(internal use for text rendering)
- [`SkinnedMeshInverseBindposes`](https://docs.rs/bevy/0.11/bevy/render/mesh/skinning/struct.SkinnedMeshInverseBindposes.html):
(internal use for skeletal animation)

These are the asset file formats (asset loaders) supported by Bevy. Support
for each one can be enabled/disabled using [cargo features](https://bevy-cheatbook.github.io/setup/bevy-config.html). Some
are enabled by default, many are not.

Image formats (loaded as [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html) assets):

| Format | Cargo feature | Default? | Filename extensions |
| --- | --- | --- | --- |
| PNG | `"png"` | Yes | `.png` |
| HDR | `"hdr"` | Yes | `.hdr` |
| KTX2 | `"ktx2"` | Yes | `.ktx2` |
| KTX2+zstd | `"ktx2", "zstd"` | Yes | `.ktx2` |
| JPEG | `"jpeg"` | No | `.jpg`, `.jpeg` |
| WebP | `"webp"` | No | `.webp` |
| OpenEXR | `"exr"` | No | `.exr` |
| TGA | `"tga"` | No | `.tga` |
| PNM | `"pnm"` | No | `.pam`, `.pbm`, `.pgm`, `.ppm` |
| BMP | `"bmp"` | No | `.bmp` |
| DDS | `"dds"` | No | `.dds` |
| KTX2+zlib | `"ktx2", "zlib"` | No | `.ktx2` |
| Basis | `"basis-universal"` | No | `.basis` |

Audio formats (loaded as [`AudioSource`](https://docs.rs/bevy/0.11/bevy/audio/struct.AudioSource.html) assets):

| Format | Cargo feature | Default? | Filename extensions |
| --- | --- | --- | --- |
| OGG Vorbis | `"vorbis"` | Yes | `.ogg`, `.oga`, `.spx` |
| FLAC | `"flac"` | No | `.flac` |
| WAV | `"wav"` | No | `.wav` |
| MP3 | `"mp3"` | No | `.mp3` |

3D asset (model or scene) formats:

| Format | Cargo feature | Default? | Filename extensions |
| --- | --- | --- | --- |
| GLTF | `"bevy_gltf"` | Yes | `.gltf`, `.glb` |

Shader formats (loaded as [`Shader`](https://docs.rs/bevy/0.11/bevy/render/render_resource/struct.Shader.html) assets):

| Format | Cargo feature | Default? | Filename extensions |
| --- | --- | --- | --- |
| WGSL | n/a | Yes | `.wgsl` |
| GLSL | `"shader_format_glsl"` | No | `.vert`, `.frag`, `.comp` |
| SPIR-V | `"shader_format_spirv"` | No | `.spv` |

Font formats (loaded as [`Font`](https://docs.rs/bevy/0.11/bevy/text/struct.Font.html) assets):

| Format | Cargo feature | Default? | Filename extensions |
| --- | --- | --- | --- |
| TrueType | n/a | Yes | `.ttf` |
| OpenType | n/a | Yes | `.otf` |

Bevy Scenes:

| Format | Filename extensions |
| --- | --- |
| RON-serialized scene | `.scn`, `.scn.ron` |

There are unofficial plugins available for adding support for even more file formats.

[Asset path labels to refer to GLTF sub-assets.](https://bevy-cheatbook.github.io/3d/gltf.html#assetpath-with-labels)

The following asset labels are supported ( `{}` is the numerical index):

- `Scene{}`: GLTF Scene as Bevy [`Scene`](https://docs.rs/bevy/0.11/bevy/scene/struct.Scene.html)
- `Node{}`: GLTF Node as [`GltfNode`](https://docs.rs/bevy/0.11/bevy/gltf/struct.GltfNode.html)
- `Mesh{}`: GLTF Mesh as [`GltfMesh`](https://docs.rs/bevy/0.11/bevy/gltf/struct.GltfMesh.html)
- `Mesh{}/Primitive{}`: GLTF Primitive as Bevy [`Mesh`](https://docs.rs/bevy/0.11/bevy/render/mesh/struct.Mesh.html)
- `Mesh{}/Primitive{}/MorphTargets`: Morph target animation data for a GLTF Primitive
- `Texture{}`: GLTF Texture as Bevy [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html)
- `Material{}`: GLTF Material as Bevy [`StandardMaterial`](https://docs.rs/bevy/0.11/bevy/pbr/struct.StandardMaterial.html)
- `DefaultMaterial`: as above, if the GLTF file contains a default material with no index
- `Animation{}`: GLTF Animation as Bevy [`AnimationClip`](https://docs.rs/bevy/0.11/bevy/animation/struct.AnimationClip.html)
- `Skin{}`: GLTF mesh skin as Bevy [`SkinnedMeshInverseBindposes`](https://docs.rs/bevy/0.11/bevy/render/mesh/skinning/struct.SkinnedMeshInverseBindposes.html)

TODO

[`wgpu`](https://github.com/gfx-rs/wgpu) (and hence Bevy) supports the following backends:

| Platform | Backends (in order of priority) |
| --- | --- |
| Linux | Vulkan, GLES3 |
| Windows | DirectX 12, Vulkan, GLES3 |
| macOS | Metal |
| iOS | Metal |
| Android | Vulkan, GLES3 |
| Web | WebGPU, WebGL2 |

On GLES3 and WebGL2, some renderer features are unsupported and performance is worse.

WebGPU is experimental and few browsers support it.

Internally, Bevy has these built-in [schedules](https://bevy-cheatbook.github.io/programming/schedules.html):

- [`Main`](https://docs.rs/bevy/0.11/bevy/app/struct.Main.html):
runs every frame update cycle, to perform general app logic
- [`ExtractSchedule`](https://docs.rs/bevy/0.11/bevy/render/struct.ExtractSchedule.html):
runs after `Main`, to copy data from the Main World into the Render World
- [`Render`](https://docs.rs/bevy/0.11/bevy/render/struct.Render.html):
runs after `ExtractSchedule`, to perform all rendering/graphics, in parallel with the next `Main` run

The `Main` schedule simply runs a sequence of other schedules:

On the first run (first frame update of the app):

- [`PreStartup`](https://docs.rs/bevy/0.11/bevy/app/struct.PreStartup.html)
- [`Startup`](https://docs.rs/bevy/0.11/bevy/app/struct.Startup.html)
- [`PostStartup`](https://docs.rs/bevy/0.11/bevy/app/struct.PostStartup.html)

On every run (controlled via the [`MainScheduleOrder`](https://docs.rs/bevy/0.11/bevy/app/struct.MainScheduleOrder.html) [resource](https://bevy-cheatbook.github.io/programming/res.html)):

- [`First`](https://docs.rs/bevy/0.11/bevy/app/struct.First.html): any initialization that must be done at the start of every frame
- [`PreUpdate`](https://docs.rs/bevy/0.11/bevy/app/struct.PreUpdate.html): for engine-internal systems intended to run before user logic
- [`StateTransition`](https://docs.rs/bevy/0.11/bevy/app/struct.StateTransition.html): perform any pending [state](https://bevy-cheatbook.github.io/programming/states.html) transitions
- [`RunFixedUpdateLoop`](https://docs.rs/bevy/0.11/bevy/app/struct.RunFixedUpdateLoop.html): runs the [`FixedUpdate`](https://docs.rs/bevy/0.11/bevy/app/struct.FixedUpdate.html) schedule as many times as needed
- [`Update`](https://docs.rs/bevy/0.11/bevy/app/struct.Update.html): for all user logic (your systems) that should run every frame
- [`PostUpdate`](https://docs.rs/bevy/0.11/bevy/app/struct.PostUpdate.html): for engine-internal systems intended to run after user logic
- [`Last`](https://docs.rs/bevy/0.11/bevy/app/struct.Last.html): any final cleanup that must be done at the end of every frame

`FixedUpdate` is for all user logic (your systems) that should run at a [fixed timestep](https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html).

`StateTransition` runs the
[`OnEnter(...)`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.OnEnter.html)/ [`OnTransition(...)`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.OnTransition.html)/ [`OnExit(...)`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.OnExit.html)
schedules for your [states](https://bevy-cheatbook.github.io/programming/states.html), when you want to change state.

The [`Render`](https://docs.rs/bevy/0.11/bevy/render/struct.Render.html) schedule is organized using [sets](https://bevy-cheatbook.github.io/programming/system-sets.html) ( [`RenderSet`](https://docs.rs/bevy/0.11/bevy/render/enum.RenderSet.html)):

- `ExtractCommands`: apply [deferred](https://bevy-cheatbook.github.io/programming/deferred.html) buffers from systems that ran in `ExtractSchedule`
- `Prepare`/ `PrepareFlush`: set up data on the GPU (buffers, textures, etc.)
- `Queue`/ `QueueFlush`: generate the render jobs to be run (usually [phase items](https://bevy-cheatbook.github.io/TODO.html))
- `PhaseSort`/ `PhaseSortFlush`: sort and batch [phase items](https://bevy-cheatbook.github.io/TODO.html) for efficient rendering
- `Render`/ `RenderFlush`: execute the [render graph](https://bevy-cheatbook.github.io/TODO.html) to actually trigger the GPU to do work
- `Cleanup`/ `CleanupFlush`: clear any data from the render World that should not persist to the next frame

The `*Flush` variants are just to apply any [deferred](https://bevy-cheatbook.github.io/programming/deferred.html) buffers after every step, if needed.

TODO

TODO

Bevy's built-in [bundle](https://bevy-cheatbook.github.io/programming/bundle.html) types, for spawning different common
kinds of entities.

[(List in API Docs)](https://docs.rs/bevy/0.11/bevy/ecs/bundle/trait.Bundle.html#implementors)

Any tuples of up to 15 [`Component`](https://docs.rs/bevy/0.11/bevy/ecs/component/trait.Component.html) types are valid bundles.

General:

- [`SpatialBundle`](https://docs.rs/bevy/0.11/bevy/render/prelude/struct.SpatialBundle.html):
Contains the required [transform](https://bevy-cheatbook.github.io/fundamentals/transforms.html) and [visibility](https://bevy-cheatbook.github.io/fundamentals/visibility.html)
components that must be included on _all_ entities that need rendering or [hierarchy](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html)
- [`TransformBundle`](https://docs.rs/bevy/0.11/bevy/transform/struct.TransformBundle.html):
Contains only the transform types, subset of `SpatialBundle`
- [`VisibilityBundle`](https://docs.rs/bevy/0.11/bevy/render/view/visibility/struct.VisibilityBundle.html):
Contains only the visibility types, subset of `SpatialBundle`

Scenes:

- [`SceneBundle`](https://docs.rs/bevy/0.11/bevy/scene/struct.SceneBundle.html):
Used for spawning scenes
- [`DynamicSceneBundle`](https://docs.rs/bevy/0.11/bevy/scene/struct.DynamicSceneBundle.html):
Used for spawning dynamic scenes

Audio:

- [`AudioBundle`](https://docs.rs/bevy/0.11/bevy/audio/type.AudioBundle.html):
Play \[audio\]\[cb::audio\] from an [`AudioSource`](https://docs.rs/bevy/0.11/bevy/audio/struct.AudioSource.html) asset
- [`SpatialAudioBundle`](https://docs.rs/bevy/0.11/bevy/audio/type.SpatialAudioBundle.html):
Play [positional audio](https://bevy-cheatbook.github.io/audio/spatial.html) from an [`AudioSource`](https://docs.rs/bevy/0.11/bevy/audio/struct.AudioSource.html) asset
- [`AudioSourceBundle`](https://docs.rs/bevy/0.11/bevy/audio/struct.AudioSourceBundle.html):
Play audio from a [custom data source/stream](https://bevy-cheatbook.github.io/audio/custom.html)
- [`SpatialAudioSourceBundle`](https://docs.rs/bevy/0.11/bevy/audio/struct.SpatialAudioSourceBundle.html):
Play positional audio from a [custom data source/stream](https://bevy-cheatbook.github.io/audio/custom.html)

Bevy 3D:

- [`Camera3dBundle`](https://docs.rs/bevy/0.11/bevy/core_pipeline/core_3d/struct.Camera3dBundle.html):
3D camera, can use perspective (default) or orthographic projection
- [`TemporalAntiAliasBundle`](https://docs.rs/bevy/0.11/bevy/core_pipeline/experimental/taa/struct.TemporalAntiAliasBundle.html):
Add this to a 3D camera to enable TAA
- [`ScreenSpaceAmbientOcclusionBundle`](https://docs.rs/bevy/0.11/bevy/pbr/struct.ScreenSpaceAmbientOcclusionBundle.html):
Add this to a 3D camera to enable SSAO
- [`MaterialMeshBundle`](https://docs.rs/bevy/0.11/bevy/pbr/struct.MaterialMeshBundle.html):
3D Object/Primitive: a Mesh and a custom Material to draw it with
- [`PbrBundle`](https://docs.rs/bevy/0.11/bevy/pbr/type.PbrBundle.html):
`MaterialMeshBundle` with the default Physically-Based Material ( [`StandardMaterial`](https://docs.rs/bevy/0.11/bevy/pbr/struct.StandardMaterial.html))
- [`DirectionalLightBundle`](https://docs.rs/bevy/0.11/bevy/pbr/struct.DirectionalLightBundle.html):
3D directional light (like the sun)
- [`PointLightBundle`](https://docs.rs/bevy/0.11/bevy/pbr/struct.PointLightBundle.html):
3D point light (like a lamp or candle)
- [`SpotLightBundle`](https://docs.rs/bevy/0.11/bevy/pbr/struct.SpotLightBundle.html):
3D spot light (like a projector or flashlight)

Bevy 2D:

- [`Camera2dBundle`](https://docs.rs/bevy/0.11/bevy/core_pipeline/core_2d/struct.Camera2dBundle.html):
2D camera, uses orthographic projection + other special configuration for 2D
- [`SpriteBundle`](https://docs.rs/bevy/0.11/bevy/sprite/struct.SpriteBundle.html):
2D sprite ( [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html) asset type)
- [`SpriteSheetBundle`](https://docs.rs/bevy/0.11/bevy/sprite/struct.SpriteSheetBundle.html):
2D sprite ( [`TextureAtlas`](https://docs.rs/bevy/0.11/bevy/sprite/struct.TextureAtlas.html) asset type)
- [`MaterialMesh2dBundle`](https://docs.rs/bevy/0.11/bevy/sprite/struct.MaterialMesh2dBundle.html):
2D shape, with custom Mesh and Material (similar to 3D objects)
- [`Text2dBundle`](https://docs.rs/bevy/0.11/bevy/text/struct.Text2dBundle.html):
Text to be drawn in the 2D world (not the UI)

Bevy UI:

- [`NodeBundle`](https://docs.rs/bevy/0.11/bevy/ui/node_bundles/struct.NodeBundle.html):
Empty node element (like HTML `<div>`)
- [`ButtonBundle`](https://docs.rs/bevy/0.11/bevy/ui/node_bundles/struct.ButtonBundle.html):
Button element
- [`ImageBundle`](https://docs.rs/bevy/0.11/bevy/ui/node_bundles/struct.ImageBundle.html):
Image element ( [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html) asset type)
- [`AtlasImageBundle`](https://docs.rs/bevy/0.11/bevy/ui/node_bundles/struct.AtlasImageBundle.html):
Image element ( [`TextureAtlas`](https://docs.rs/bevy/0.11/bevy/sprite/struct.TextureAtlas.html) asset type)
- [`TextBundle`](https://docs.rs/bevy/0.11/bevy/ui/node_bundles/struct.TextBundle.html):
Text element

[(more info about working with resources)](https://bevy-cheatbook.github.io/programming/res.html)

These resources allow you to change the settings for how various parts of Bevy work.

These may be inserted at the start, but should also be fine to change at runtime (from a
[system](https://bevy-cheatbook.github.io/programming/systems.html)):

- [`ClearColor`](https://docs.rs/bevy/0.11/bevy/core_pipeline/clear_color/struct.ClearColor.html):
Global renderer background color to clear the window at the start of each frame
- [`GlobalVolume`](https://docs.rs/bevy/0.11/bevy/audio/struct.GlobalVolume.html):
The overall volume for playing audio
- [`AmbientLight`](https://docs.rs/bevy/0.11/bevy/pbr/struct.AmbientLight.html):
Global renderer "fake lighting", so that shadows don't look too dark / black
- [`Msaa`](https://docs.rs/bevy/0.11/bevy/render/view/enum.Msaa.html):
Global renderer setting for Multi-Sample Anti-Aliasing (some platforms might only support the values 1 and 4)
- [`UiScale`](https://docs.rs/bevy/0.11/bevy/ui/struct.UiScale.html):
Global scale value to make all UIs bigger/smaller
- [`GizmoConfig`](https://docs.rs/bevy/0.11/bevy/gizmos/struct.GizmoConfig.html):
Controls how [gizmos](https://bevy-cheatbook.github.io/fundamentals/gizmos.html) are rendered
- [`WireframeConfig`](https://docs.rs/bevy/0.11/bevy/pbr/wireframe/struct.WireframeConfig.html):
Global toggle to make everything be rendered as wireframe
- [`GamepadSettings`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.GamepadSettings.html):
Gamepad input device settings, like joystick deadzones and button sensitivities
- [`WinitSettings`](https://docs.rs/bevy/0.11/bevy/winit/struct.WinitSettings.html):
Settings for the OS Windowing backend, including update loop / power-management settings
- [`TimeUpdateStrategy`](https://docs.rs/bevy/0.11/bevy/time/enum.TimeUpdateStrategy.html):
Used to control how the [`Time`](https://docs.rs/bevy/0.11/bevy/time/struct.Time.html) is updated
- [`Schedules`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.Schedules.html):
Stores all [schedules](https://bevy-cheatbook.github.io/programming/schedules.html), letting you register additional functionality at runtime
- [`MainScheduleOrder`](https://docs.rs/bevy/0.11/bevy/app/struct.MainScheduleOrder.html):
The sequence of [schedules](https://bevy-cheatbook.github.io/programming/schedules.html) that will run every frame update

Settings that are not modifiable at runtime are not represented using resources. Instead,
they are configured via the respective [plugins](https://bevy-cheatbook.github.io/builtins.html#plugins).

These resources provide access to different features of the game engine at runtime.

Access them from your [systems](https://bevy-cheatbook.github.io/programming/systems.html), if you need their state, or to control the respective
parts of Bevy. These resources are in the [Main World](https://bevy-cheatbook.github.io/gpu/intro.html). [See here for the\\
resources in the Render World](https://bevy-cheatbook.github.io/builtins.html#render-world).

- [`Time`](https://docs.rs/bevy/0.11/bevy/time/struct.Time.html):
Global time-related information (current frame delta time, time since startup, etc.)
- [`FixedTime`](https://docs.rs/bevy/0.11/bevy/time/fixed_timestep/struct.FixedTime.html):
Tracks remaining time until the next [fixed update](https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html)
- [`AssetServer`](https://docs.rs/bevy/0.11/bevy/asset/struct.AssetServer.html):
Control the asset system: Load assets, check load status, etc.
- [`Assets<T>`](https://docs.rs/bevy/0.11/bevy/asset/struct.Assets.html):
Contains the actual data of the loaded assets of a given type
- [`State<T>`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.State.html):
The current value of a [states type](https://bevy-cheatbook.github.io/programming/states.html)
- [`NextState<T>`](https://docs.rs/bevy/0.11/bevy/ecs/schedule/struct.NextState.html):
Used to queue a transition to another [state](https://bevy-cheatbook.github.io/programming/states.html)
- [`Gamepads`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.Gamepads.html):
Tracks the IDs for all currently-detected (connected) gamepad devices
- [`SceneSpawner`](https://docs.rs/bevy/0.11/bevy/scene/struct.SceneSpawner.html):
Direct control over spawning Scenes into the main app World
- [`FrameCount`](https://docs.rs/bevy/0.11/bevy/core/struct.FrameCount.html):
The total number of frames
- [`ScreenshotManager`](https://docs.rs/bevy/0.11/bevy/render/view/window/screenshot/struct.ScreenshotManager.html):
Used to request a screenshot of a window to be taken/saved
- [`AppTypeRegistry`](https://docs.rs/bevy/0.11/bevy/ecs/reflect/struct.AppTypeRegistry.html):
Access to the Reflection Type Registry
- [`AsyncComputeTaskPool`](https://docs.rs/bevy/0.11/bevy/tasks/struct.AsyncComputeTaskPool.html):
Task pool for running background CPU tasks
- [`ComputeTaskPool`](https://docs.rs/bevy/0.11/bevy/tasks/struct.ComputeTaskPool.html):
Task pool where the main app schedule (all the systems) runs
- [`IoTaskPool`](https://docs.rs/bevy/0.11/bevy/tasks/struct.IoTaskPool.html):
Task pool where background i/o tasks run (like asset loading)
- [`WinitWindows`](https://docs.rs/bevy/0.11/bevy/winit/struct.WinitWindows.html) ( [non-send](https://bevy-cheatbook.github.io/programming/non-send.html)):
Raw state of the `winit` backend for each window
- [`NonSendMarker`](https://docs.rs/bevy/0.11/bevy/render/view/struct.NonSendMarker.html):
Dummy resource to ensure a system always runs on the main thread

These resources are present in the [Render World](https://bevy-cheatbook.github.io/gpu/intro.html). They can be accessed
from rendering systems (that run during [render stages](https://bevy-cheatbook.github.io/gpu/stages.html)).

- [`MainWorld`](https://docs.rs/bevy/0.11/bevy/render/struct.MainWorld.html):
(extract schedule only!) access data from the Main World
- [`RenderGraph`](https://docs.rs/bevy/0.11/bevy/render/render_graph/struct.RenderGraph.html):
[The Bevy Render Graph](https://bevy-cheatbook.github.io/TODO.html)
- [`PipelineCache`](https://docs.rs/bevy/0.11/bevy/render/render_resource/struct.PipelineCache.html):
Bevy's manager of render pipelines. Used to store render pipelines used by the app, to avoid
recreating them more than once.
- [`TextureCache`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.TextureCache.html):
Bevy's manager of temporary textures. Useful when you need textures to use internally
during rendering.
- [`DrawFunctions<P>`](https://docs.rs/bevy/0.11/bevy/render/render_phase/struct.DrawFunctions.html):
Stores draw functions for a given phase item type
- [`RenderAssets<T>`](https://docs.rs/bevy/0.11/bevy/render/render_asset/struct.RenderAssets.html):
Contains handles to the GPU representations of currently loaded asset data
- [`DefaultImageSampler`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.DefaultImageSampler.html):
The default sampler for [`Image`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.Image.html) asset textures
- [`FallbackImage`](https://docs.rs/bevy/0.11/bevy/render/texture/struct.FallbackImage.html):
Dummy 1x1 pixel white texture. Useful for shaders that normally need a texture, when
you don't have one available.

There are many other resources in the Render World, which are not mentioned
here, either because they are internal to Bevy's rendering algorithms, or
because they are just extracted copies of the equivalent resources in the Main
World.

Using these resources, you can have direct access to the `wgpu` APIs for controlling the GPU.
These are available in both the Main World and the Render World.

- [`RenderDevice`](https://docs.rs/bevy/0.11/bevy/render/renderer/struct.RenderDevice.html):
The GPU device, used for creating hardware resources for rendering/compute
- \[ `RenderQueue`\]\[bevy::RenderQueue\]:
The GPU queue for submitting work to the hardware
- [`RenderAdapter`](https://docs.rs/bevy/0.11/bevy/render/renderer/struct.RenderAdapter.html):
Handle to the physical GPU hardware
- [`RenderAdapterInfo`](https://docs.rs/bevy/0.11/bevy/render/renderer/struct.RenderAdapterInfo.html):
Information about the GPU hardware that Bevy is running on

These resources represent the current state of different input devices. Read them from your
[systems](https://bevy-cheatbook.github.io/programming/systems.html) to [handle user input](https://bevy-cheatbook.github.io/input.html).

- [`Input<KeyCode>`](https://docs.rs/bevy/0.11/bevy/input/keyboard/enum.KeyCode.html):
Keyboard key state, as a binary [Input](https://docs.rs/bevy/0.11/bevy/input/struct.Input.html) value
- [`Input<MouseButton>`](https://docs.rs/bevy/0.11/bevy/input/mouse/enum.MouseButton.html):
Mouse button state, as a binary [Input](https://docs.rs/bevy/0.11/bevy/input/struct.Input.html) value
- [`Input<GamepadButton>`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.GamepadButton.html):
Gamepad buttons, as a binary [Input](https://docs.rs/bevy/0.11/bevy/input/struct.Input.html) value
- [`Axis<GamepadAxis>`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.GamepadAxis.html):
Analog [Axis](https://docs.rs/bevy/0.11/bevy/input/struct.Axis.html) gamepad inputs (joysticks and triggers)
- [`Axis<GamepadButton>`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.GamepadButton.html):
Gamepad buttons, represented as an analog [Axis](https://docs.rs/bevy/0.11/bevy/input/struct.Axis.html) value
- [`Touches`](https://docs.rs/bevy/0.11/bevy/input/touch/struct.Touches.html):
The state of all fingers currently touching the touchscreen
- [`Gamepads`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.Gamepads.html):
Registry of all the connected [`Gamepad`](https://docs.rs/bevy/0.11/bevy/input/gamepad/struct.Gamepad.html) IDs

[(more info about working with events)](https://bevy-cheatbook.github.io/programming/events.html)

These [events](https://bevy-cheatbook.github.io/programming/events.html) fire on activity with input devices. Read them to \[handle user input\]\[cb::input\].

- [`MouseButtonInput`](https://docs.rs/bevy/0.11/bevy/input/mouse/struct.MouseButtonInput.html):
Changes in the state of mouse buttons
- [`MouseWheel`](https://docs.rs/bevy/0.11/bevy/input/mouse/struct.MouseWheel.html):
Scrolling by a number of pixels or lines ( [`MouseScrollUnit`](https://docs.rs/bevy/0.11/bevy/input/mouse/enum.MouseScrollUnit.html))
- [`MouseMotion`](https://docs.rs/bevy/0.11/bevy/input/mouse/struct.MouseMotion.html):
Relative movement of the mouse (pixels from previous frame), regardless of the OS pointer/cursor
- [`CursorMoved`](https://docs.rs/bevy/0.11/bevy/window/struct.CursorMoved.html):
New position of the OS mouse pointer/cursor
- [`KeyboardInput`](https://docs.rs/bevy/0.11/bevy/input/keyboard/struct.KeyboardInput.html):
Changes in the state of keyboard keys (keypresses, not text)
- [`ReceivedCharacter`](https://docs.rs/bevy/0.11/bevy/window/struct.ReceivedCharacter.html):
Unicode text input from the OS (correct handling of the user's language and layout)
- [`Ime`](https://docs.rs/bevy/0.11/bevy/window/enum.Ime.html):
Unicode text input from IME (support for advanced text input in different scripts)
- [`TouchInput`](https://docs.rs/bevy/0.11/bevy/input/touch/struct.TouchInput.html):
Change in the state of a finger touching the touchscreen
- [`GamepadEvent`](https://docs.rs/bevy/0.11/bevy/input/gamepad/enum.GamepadEvent.html):
Changes in the state of a gamepad or any of its buttons or axes
- [`GamepadRumbleRequest`](https://docs.rs/bevy/0.11/bevy/input/gamepad/enum.GamepadRumbleRequest.html):
Send these events to control gamepad rumble
- [`TouchpadMagnify`](https://docs.rs/bevy/0.11/bevy/input/touchpad/struct.TouchpadMagnify.html):
Pinch-to-zoom gesture on laptop touchpad (macOS)
- [`TouchpadRotate`](https://docs.rs/bevy/0.11/bevy/input/touchpad/struct.TouchpadRotate.html):
Two-finger rotate gesture on laptop touchpad (macOS)

[Events](https://bevy-cheatbook.github.io/programming/events.html) related to various internal things happening during the
normal runtime of a Bevy app.

- [`AssetEvent<T>`](https://docs.rs/bevy/0.11/bevy/asset/enum.AssetEvent.html):
Sent by Bevy when [asset data](https://bevy-cheatbook.github.io/assets.html) has been added/modified/removed; [can be used to detect changes to assets](https://bevy-cheatbook.github.io/assets/assetevent.html)
- [`HierarchyEvent`](https://docs.rs/bevy/0.11/bevy/hierarchy/enum.HierarchyEvent.html):
Sent by Bevy when entity [parents/children](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html) change
- [`AppExit`](https://docs.rs/bevy/0.11/bevy/app/struct.AppExit.html):
Tell Bevy to shut down

Events from the OS / windowing system, or to control Bevy.

- [`RequestRedraw`](https://docs.rs/bevy/0.11/bevy/window/struct.RequestRedraw.html):
In an app that does not refresh continuously, request one more update before going to sleep
- [`FileDragAndDrop`](https://docs.rs/bevy/0.11/bevy/window/enum.FileDragAndDrop.html):
The user drag-and-dropped a file into our app
- [`CursorEntered`](https://docs.rs/bevy/0.11/bevy/window/struct.CursorEntered.html):
OS mouse pointer/cursor entered one of our windows
- [`CursorLeft`](https://docs.rs/bevy/0.11/bevy/window/struct.CursorLeft.html):
OS mouse pointer/cursor exited one of our windows
- [`WindowCloseRequested`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowCloseRequested.html):
OS wants to close one of our windows
- [`WindowCreated`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowCreated.html):
New application window opened
- [`WindowClosed`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowClosed.html):
Bevy window closed
- [`WindowDestroyed`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowDestroyed.html):
OS window freed/dropped after window close
- [`WindowFocused`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowFocused.html):
One of our windows is now focused
- [`WindowMoved`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowMoved.html):
OS/user moved one of our windows
- [`WindowResized`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowResized.html):
OS/user resized one of our windows
- [`WindowScaleFactorChanged`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowScaleFactorChanged.html):
One of our windows has changed its DPI scaling factor
- [`WindowBackendScaleFactorChanged`](https://docs.rs/bevy/0.11/bevy/window/struct.WindowBackendScaleFactorChanged.html):
OS reports change in DPI scaling factor for a window

The complete list of individual component types is too specific to be useful to list here.

See: [(List in API Docs)](https://docs.rs/bevy/0.11/bevy/ecs/component/trait.Component.html#implementors)

[GitHub Sponsors](https://github.com/sponsors/inodentry) [Patreon](https://patreon.com/iyesgames) [Bitcoin](bitcoin:bc1qaf32uqsg6mngw9g4aqc3l2jvuv46qx0zw2438p) If you like this book, please donate to support me!

I also offer professional tutoring / private lessons for Bevy and Rust. [Contact me](https://bevy-cheatbook.github.io/contact.html) if interested!