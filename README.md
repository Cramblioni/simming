# simming
## A simple implementation of the approach used for autonomy used in "The Sims"

The system has two items, sims(monomorphic) and interactables(polymorphic).

On each cycle the world requests an advert from each interactable, it collects
them and then updates each sim using that list of adverts.

On being updated, Each sim will:
    - Periodically decay stats
    - one of
        - continue enacting (if is enacting)
        - enact from queue (if not enacting and task queued)
        - satisfy stats (if stats given a condition)

## TODO
- [ ] Cancellable Enactment
    - [ ] Effect evaluation (to compare the effects of continuing versus
          cancelling)
    - [ ] Priority
- [ ] More stats
    - [ ] Bladder
    - [ ] Fun
    - [ ] Social?
- [ ] More Objects
    ...
- [ ] Simulation Constraints
    - [ ] Rooms
    - [ ] movement (explained in ideas section under movement)
        - [ ] internal (in room) 
        - [ ] external (outside room)
- [ ] graphics

## IDEAS

### movement
When it comes to pathfinding, My idea is to split it up into two parts, internal
and external. Internal pathfinding would be pathfinding in a room. This is a
simple grid-based pathfinding thingy. External pathfinding is pathfinding
between rooms. For example, if a sim needs to pathfind from the living room to
an object in the kitchen, they'll find a path on the room level, doing internal
pathfinding for the room they're currently in.
This is an aesthetic choice, because it would allow us to use different
pathfinding approaches for each level. So a sim will find the shortest path when
walking through a room, but will employ a gradual backtracking based search for
room to room traversal.
This could reduce grid sizes, but does depend on a room graph being simple to
clone and then modify.
The graph should be constructed based on the passage zones between rooms,
meaning multiple edges between rooms can exist, so distances between them should
be kept in mind.
Pathfinding should deal with pathfinding to an area, not just a specific point.
