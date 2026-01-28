use std::{cmp::{max, min}, collections::HashSet};

use log::info;
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator};
use specs::Entity;

use crate::generate::rect::Rect;

// Room constants
pub const MIN_SIZE: i32 = 8;
pub const MAX_SIZE: i32 = 16;
pub const MAX_ROOMS: i32 = 25;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
    Debris,
}

impl TileType {
    pub const fn description(&self) -> &'static str {
        match self {
           TileType::Wall => "It's a chunk of rough wall.",
           TileType::Floor => "It's a patch of rough ground.",
           TileType::DownStairs => "A narrow staircase leading further downwards.",
           TileType::UpStairs => "A narrow staircase leading further upwards.",
           TileType::Debris => "A large pile of rocks, too big to climb over",
        }
    }
}

pub struct MapOptions {
    pub width: i32,
    pub height: i32,
    pub has_upstairs: bool,
    pub has_downstairs: bool,
    pub has_debris: bool,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub tile_content: Vec<Vec<Entity>>,
    pub revealed_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub bloodstains: HashSet<usize>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub player_spawn_index: Option<usize>,
}

impl Map {
    /**
     * Given a position tuple, returns the index offset of that position
     * using the single array structured map.
     */
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    /**
     * The reverse of the `xy_idx()` function above.
     */
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        let x = (idx % (self.width as usize)) as i32;
        let y = (idx / (self.width as usize)) as i32;
        (x, y)
    }

    /**
     * Mutates the tiles of the given map to have floors
     * corresponding to the provided room.
     */
    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in (room.y1 + 1) ..= room.y2 {
            for x in (room.x1 + 1) ..= room.x2 {
                let index = self.xy_idx(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }
    
    fn apply_horizontal_tunnel(&mut self, width: usize, height: usize, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < width * height {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_vertical_tunnel(&mut self, width: usize, height: usize, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < width * height {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(& self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked_tiles[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked_tiles[index] = match *tile {
                TileType::Wall | TileType::Debris => true,
                _ => false,
            }
        }
    }

    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Procedurally creates a new map instance of oakwood, the starting village.
    pub fn new_map_oakwood(_rng: &mut RandomNumberGenerator, options: MapOptions) -> Map {
        let width: usize = options.width as usize;
        let height: usize = options.height as usize;

        let mut map = Map {
            tiles: vec![TileType::Floor; (width as usize) * (height as usize)],
            tile_content: vec![Vec::new(); (width as usize) * (height as usize)],
            revealed_tiles: vec![false; (width as usize) * (height as usize)],
            blocked_tiles: vec![false; (width as usize) * (height as usize)],
            bloodstains: HashSet::new(),
            rooms: Vec::new(),
            width: options.width,
            height: options.height,
            player_spawn_index: None,
        };

        // Overwrite the map borders with walls
        for (index, tile) in map.tiles.iter_mut().enumerate() {
            let (x, y) = fn_idx_xy(width, index);
            if x == 0 || y == 0  || x == options.width - 1 || y == options.height - 1 {
                *tile = TileType::Wall;
            }
        }

        // Create a basic house
        for x in 35..=45 {
            for y in 5..=15 {
                // door
                if [39, 40, 41].contains(&x) && y == 15 {
                    continue;
                }
                // inside
                if x >= 36 && x <= 44 && y >= 6 && y <= 14 {
                    continue;
                }
                let index = map.xy_idx(x, y);
                map.tiles[index] = TileType::Wall;
            }
        }

        map.player_spawn_index = Some(map.xy_idx(40, 20));

        let downstairs_index = map.xy_idx(40, 30);
        map.tiles[downstairs_index] = TileType::DownStairs;

        return map;        
    }
    
    pub fn new_map_dynamic_rooms_and_corridors(rng: &mut RandomNumberGenerator, options: MapOptions) -> Map {
        let width: usize = options.width as usize;
        let height: usize = options.height as usize;

        let mut map = Map {
            tiles: vec![TileType::Wall; (width as usize) * (height as usize)],
            tile_content: vec![Vec::new(); (width as usize) * (height as usize)],
            revealed_tiles: vec![false; (width as usize) * (height as usize)],
            blocked_tiles: vec![false; (width as usize) * (height as usize)],
            bloodstains: HashSet::new(),
            rooms: Vec::new(),
            width: options.width,
            height: options.height,
            player_spawn_index: None,
        };
    
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, options.width - w - 1) - 1;
            let y = rng.roll_dice(1, options.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(width, height, prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(width, height, prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(width, height, prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(width, height, prev_x, new_x, new_y);
                    }
                }
                info!("Spawned new room with position ({}, {}) - ({}, {})", new_room.x1, new_room.y1, new_room.x2, new_room.y2);
                map.rooms.push(new_room);            
            }
        }

        if options.has_upstairs {
            let (upstair_x, upstair_y) = map.rooms[0].center();
            let upstair_index = map.xy_idx(upstair_x, upstair_y);
            map.tiles[upstair_index] = TileType::UpStairs;
        }

        if options.has_downstairs {
            let (downstair_x, downstair_y) = map.rooms[map.rooms.len() - 1].center();
            let downstair_index = map.xy_idx(downstair_x, downstair_y);
            map.tiles[downstair_index] = TileType::DownStairs;
        }

        if options.has_debris {
            for room in map.rooms.iter() {
                for _ in 0..rng.range(0, 7) {
                    let debris_x = rng.range(room.x1, room.x2);
                    let debris_y = rng.range(room.y1, room.y2);
                    let debris_index = map.xy_idx(debris_x, debris_y);
                    match map.tiles[debris_index] {
                        TileType::Floor => map.tiles[debris_index] = TileType::Debris,
                        _ => {},
                    }
                }
            }
        }

        map.player_spawn_index = Some(map.xy_idx(
            map.rooms[0].center().0, 
            map.rooms[0].center().1,
        ));

        // TODO: Ensure map is solvable
        return map;
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        return rltk::DistanceAlg::Pythagoras.distance2d(p1, p2);
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn _fn_xy_idx(width: usize, x: i32, y: i32) -> usize {
    (y as usize * width) + x as usize
}

pub fn fn_idx_xy(width: usize, idx: usize) -> (i32, i32) {
    let x = (idx % (width as usize)) as i32;
    let y = (idx / (width as usize)) as i32;
    (x, y)
}
