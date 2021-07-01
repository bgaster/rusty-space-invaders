# Rusty Space Invaders

An emulation of the original space invaders. The behaviour somewhat follows that described by Chris Cantrell at [Computer Archeology](https://computerarcheology.com/A.rcade/SpaceInvaders/). Below is the standard screen shot of some game play.

![](./assets/space_invaders_sample_play.gif)

Another useful place for details of how the game works can be found [here](https://www.classicgaming.cc/classics/space-invaders/play-guide).
         
It is by no means a direct reproduction, in particular, the timing is similar, but I've not made any real effort to match it precisely. It is not implemented as per the original game, using the 2 screen interrupts and so on, instead it uses a simply timer based system for the main alien swarm and other elements, e.g. bullets, just run at their own rate, both for animations, and for movement. It does not use an ECS, which if I was doing anything more complicated it would. Next project I plan to, but this was just a small few days project, while I sat around complaining about a horrid tooth ache, which has now been fixed with a root canal :-)

It is worth noting that the main goal of the project is to port it to the 32blit, a small 32-bit MCU based retro games console, which I backed on Kickstarter and should be arriving soon. More details of this here:
 
[32blit: retro-inspired handheld with open-source firmware](https://www.kickstarter.com/projects/pimoroni/32blit-retro-inspired-handheld-with-open-source-fi)

## TODOs

- [x] Hi score
   - [x] Configuration, so highscore (and version) are persistent across execution instances of game
- [x] Splash Screen
- [ ] Documentation
   - [ ] Document code
   - [x] README.md
- [ ] UFOs
   - [x] Design Sprite
   - [x] Animation
   - [x] Random spawning
   - [ ] Randomise amount of points when killed
   - [X] Bullet kill
   - [X] Fire bullet when on screen
   - [x] Sound effect
- Additional player live(s) when passed certain score
- [x] Gamepad support
- [ ] Alien invasion
- [x] Barriers
   - [x] Sprites
   - [x] Basic Rendering
   - [x] Collisions
      - [x] Full bounding box
      - [x] Partial damage
- [x] End of game
   - [x] Functionality to reset game
   - [x] Screen showing end of game text
   - [x] Timer to provide delay between end of game and new game/splash screen
- [x] Levels
   - [x] Next level functionality
   - [x] Timer to slightly delay next level starting
- [x] Sound
   - [x] Sound effects for player, alien, and bullets
   - [x] Sound track
      - [x] Ableton to generate 80, 100, 120, and so BPM loops for different speed of alien swarms
      - [x] Intergrate music into game
      - [x] Change music tempo when aliens die
- [ ] Port to Raspberry Pi
   - [ ] Raylib backend
- [x] Push to git hub

## Rust

It is implemented using [Rust](https://www.rust-lang.org/), mostly because I like it and I'm trying to explore different ideas to better understand the language and game programming in general.

The code base is very simple and builds with stable, I used 1.41.0 (5e1a79984 2020-01-27), on OS X. It should build on any platform, although I have not tested it and it does require Portaudio. There are a few things that could be fixed, in particular, a couple of timing 'hacks' and as some sprites are animated, e.g. player explosion, and some not, a few places an Either type is used to allow either type to be used. I suppose this should really be abstracted out into a trait, maybe some day :-).

There are a few more dependencies than I would like, as I plan to move it to embedded and no std, this will likely change once I finally get the 32blit delivered. For now I don't suppose it is a huge issue.

To install Rust go you need simply to install [Rustup](https://rustup.rs/) and if you already have Rust installed, then you can update with the command rustup update.

## Building

Currently everything has to be built from source. To build run the command:

```bash
cargo build --release
```

Start the game with:

```bash
cargo run --release
```

## Assets

The assets are all sprite based, of varying sizes, although all pretty small. They drawn using
the super cool Rust pixel editor [Rx](https://github.com/cloudhead/rx), and then, in some cases, tweaked in Photoshop, and finally added to a single sprite sheet with [TexturePacker](https://www.codeandweb.com/texturepacker).

There is a simple sprite sheet and animation engine, which provides just want was needed for this game. The main reason for not using an existing Rust game engine was, as noted above, that I want to port it to the 32Blit, which would be a lot more work if I used a complicated cross platform engine. I do use a few crates when I could assume that there will be similar ones on the 32blit, even though there they will be in C++ (ahh) and I'll have to implement abstractions on top. In particular, eculid, and pixels for math and a framebuffer, respectively. Along with wint for windowing stuff. For other desktop games I plan to use [rgfx](https://github.com/cloudhead/rgx), which looks great.

## Credits

Thanks to Alexis Sellier (aka cloudhead) for the great pixel editor tool, [Rx](https://github.com/cloudhead/rx).

Also I was, in part, inspired to write this game after watching Catherine West's RustConf'18 [keynote](https://www.youtube.com/watch?v=P9u8x13W7UE), where she talks about ECS implementations in Rust. In truth I'd not spent much time looking into ECSs before now and, as noted, while this code base does not use one, I've now read lots about them, played around with Specs, and more. I'm beginning to think about what such as system might look like for an MCU based system, such as the 32blit. What I really want is a tiny ECS that is very low overhead, both from a use case point of view, but also with respect to performance overhead.

## LICENSE

The sound effects for the player bullets, player explosions, and alien explosions are from [Classics Space Invaders](https://www.classicgaming.cc/classics/space-invaders/sounds). The music for the aliens
marching is made specifically and does not match 100% to the original.

Licensed under any of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
    Mozilla Public License 2.0

at your option.

Dual MIT/Apache2 is strictly more permissive.

