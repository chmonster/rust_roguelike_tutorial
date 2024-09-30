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

## Distinctive features (TODOs)
### Visuals
- Unicode (post-ASCII) tile art
  - custom tiles have already been widely explored
  - but CP437 is a cliche
- extending the grid
  - Grid edge topological connections and wraparounds
  - Torus, Platonic solids (cube, dodecahedron, icosahedron), Sphere, Dyson sphere, Klein bottle, Pyramid
  - Random terrain generation integrated with zone theme
- Word balloon conversations
  - tooltip-like
  - possibly with crpg-type conversation options
### Entities
- Doors leading to unconnected spaces (ie non random teleport)
- different takes on rpg mobs, races, loot etc.
  - Gold not taken as currency?
  - Guns, parachutes, vehicles?
- item use not necessarily revealed on acquisition
  - avoiding rogue/rpg item cliches
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
  - can cookies, browser storage
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
    - npcs refer to literary/philosophical figures (as the pc does)
    - while still alluding to rogue/rpg stereotypes (dragons, demons etc)
  ### Plot
  - character starts in a little room
  - leaves and enters a series of weird levels
  - travels through zones - multiple levels within a zone sharing topology and features
    - map topology is plot appropriate
    - figuring out how the map works is part of the game
    - unique items at end of zone
      - matching zone shape - orb, D20, magic donut
      - opening new UI features, stat knowledge, etc, instead of just new abilities
      - effectively gates to get to the next zone
  - at the end, transcendence
    - replayability options granted?
