# Working title: *Kafka*

## General description
- 2D roguelike
- Turn-based, tile-based, procedurally generated
- Experimental 
  - deviates from, challenges classic rogue conventions
- Self-aware post-modern vibe
  - Not solely dungeon or fantasy based
  - self-referential
  - semi-realistic, as much as an @ on a CRT can be
- Coded in Rust
  - Extending Rtlk or installing something with more features
  - designed as WASM/browser first if possible
    - but with standalone executables available also

## Distinctive features (TODOs)
### Visuals
- Unicode (post-ASCII) tile art
  - custom tiles have already been widely explored
  - but CP437 is a cliche
  - Requires elbow-deep work in Rltk
- extending the grid
  - Grid edge topological connections and wraparounds
  - Torus, Platonic solids (cube, dodecahedron, icosahedron), Sphere, Dyson sphere, Klein bottle, Pyramid
  - Random terrain generation integrated with zone theme
- Word balloon conversations
  - tooltip-like
  - possibly with crpg-type conversation options (Baldur's Gate)
### Entities
- Doors leading to unconnected spaces (ie non random teleport)
  - Let's Make a Deal room
- What else can traps, walls, water do?
  - shrinking room, walls move in after each move
- different takes on rpg mobs, races, loot etc.
  - Gold not taken as currency?
  - Guns, parachutes, vehicles?
  - Mobs have different personalities (components) that help determine AI
    - eg cowardly vs brave
    - not visible in mob name (except with a magic item?), only by behaviour
    - rooms can generate same-typed mobs with a set distribution (eg 70% cowardly, 30% brave)
- item use not necessarily revealed on acquisition
  - avoiding rogue/rpg item cliches  
- Mobs operating outside turn cycle - pseudo-realtime gameplay
  - eg random swarm, acting each tick rather than each turn
  - use sparingly
- Use of chaos as appropriate to plot/progression
### Controls
- Minimalist controls and UI
  - WASM/browser
    - smaller subset of keys used
    - fewer available anyway (limitation of Rltk or WASM?)
  - intuitive as possible
    - only abstract information at the beginning
      - health bar without numbers, general status updates
      - zone items reveal more and more about rpg stats, percentages, what's under the hood
      - avoids traditional rogue controls
  - revealed during gameplay
    - control availability as plot rewards
- Extend mouse controls from tutorial targeting gui
  - Right clicking on an entity gives interaction options
    - replacing complex key assignments for many game options
    - ID, stats, fire weapon/spell
    - rapid path to location
### Game saves and loads
- WASM is sandboxed
  - are cookies, browser storage possible
    - clearing cookies will delete character progress
  - external save import/export files?

## Narrative/Vibe
  ### Characters
  - pc is a blank slate
    - generic at start
      - evocative of a Kafka main character, or Winston Smith from 1984
    - abilities and stats randomized but not known at start
      - incrementally revealed during gameplay
  - zone bosses/npcs talk about progress meta-textually
    - npcs refer to literary/philosophical figures (as the pc does) or even pop culture
      - Socrates, Sartre, Faust, ~~Mickey Mouse~~
      - definitely a talking Minotaur for the maze
      - Kafka final boss?
    - while still alluding to rogue/rpg stereotypes (dragons, demons etc)
  ### Plot
  - character starts in a little room
  - leaves and enters a series of weird levels
    - linear with side missions
  - travels through zones - multiple levels within a zone sharing topology and features
    - map topology is plot appropriate
    - figuring out how the map works is part of the game
    - unique items at end of zone
      - matching zone shape - orb, D20, magic donut
      - opening new UI features, stat knowledge, etc, instead of just new abilities
      - effectively gates to get to the next zone 
        - magic key but you have to do something other than opening a door
          - jump, fly, zap, eat?
  - self-aware hero's journey
  - at the end, transcendence
    - replayability options granted?
