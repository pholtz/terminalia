use std::cmp::{min, max};

use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator};

use crate::rect::Rect;

pub const MAX_WIDTH: i32 = 80;
pub const MAX_HEIGHT: i32 = 50;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

/**
 * Given a position tuple, returns the index offset of that position
 * using the single array structured map.
 */
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * MAX_WIDTH as usize) + x as usize
}

/**
 * The reverse of the `xy_idx()` function above.
 */
pub fn idx_xy(idx: usize) -> (i32, i32) {
    let x = (idx % (MAX_WIDTH as usize)) as i32;
    let y = (idx / (MAX_WIDTH as usize)) as i32;
    (x, y)
}

impl Map {
    /**
     * Mutates the tiles of the given map to have floors
     * corresponding to the provided room.
     */
    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in (room.y1 + 1) ..= room.y2 {
            for x in (room.x1 + 1) ..= room.x2 {
                self.tiles[xy_idx(x, y)] = TileType::Floor;
            }
        }
    }
    
    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = xy_idx(x, y);
            if idx > 0 && idx < 80*50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = xy_idx(x, y);
            if idx > 0 && idx < 80*50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    pub fn new_map_dynamic_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            revealed_tiles: vec![false; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            rooms: Vec::new(),
            width: MAX_WIDTH,
            height: MAX_HEIGHT,
        };
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
    
        let mut rng = RandomNumberGenerator::new();
    
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);            
            }
        }
        return map;
    }
    
    pub fn new_map_static_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            revealed_tiles: vec![false; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            rooms: Vec::new(),
            width: MAX_WIDTH,
            height: MAX_HEIGHT,
        };
        let room1 = Rect::new(20, 15, 10, 15);
        let room2 = Rect::new(35, 15, 10, 15);
        map.apply_room_to_map(&room1);
        map.apply_room_to_map(&room2);
        map.apply_horizontal_tunnel(25, 40, 23);
        return map;
    }
    
    pub fn new_map_random_walls() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Floor; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            revealed_tiles: vec![false; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            rooms: Vec::new(),
            width: MAX_WIDTH,
            height: MAX_HEIGHT,
        };
    
        // Make the boundaries walls
        for x in 0..MAX_WIDTH {
            map.tiles[xy_idx(x, 0)] = TileType::Wall;
            map.tiles[xy_idx(x, MAX_HEIGHT - 1)] = TileType::Wall;
        }
        for y in 0..MAX_HEIGHT {
            map.tiles[xy_idx(0, y)] = TileType::Wall;
            map.tiles[xy_idx(MAX_WIDTH - 1, y)] = TileType::Wall;
        }
    
        // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
        // First, obtain the thread-local RNG:
        let mut rng = rltk::RandomNumberGenerator::new();
    
        for _i in 0..25 {
            let x = rng.roll_dice(1, MAX_WIDTH - 1);
            let y = rng.roll_dice(1, MAX_HEIGHT - 1);
            let idx = xy_idx(x, y);
            if idx != xy_idx(MAX_WIDTH / 2, MAX_HEIGHT / 2) {
                map.tiles[idx] = TileType::Wall;
            }
        }
    
        return map;
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
