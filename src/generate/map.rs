use std::{cmp::{max, min}, collections::HashSet};

use log::info;
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator};
use specs::Entity;

use crate::generate::rect::Rect;

// Map constants
pub const MAX_WIDTH: i32 = 80;
pub const MAX_HEIGHT: i32 = 50;

// Room constants
pub const MIN_SIZE: i32 = 6;
pub const MAX_SIZE: i32 = 10;
pub const MAX_ROOMS: i32 = 30;
pub const MAX_MONSTERS: i32 = 20;
pub const MAX_ITEMS: i32 = 10;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor, DownStairs
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

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = xy_idx(x, y);
        !self.blocked_tiles[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked_tiles[index] = *tile == TileType::Wall;
        }
    }

    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
    
    pub fn new_map_dynamic_rooms_and_corridors(rng: &mut RandomNumberGenerator) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            tile_content: vec![Vec::new(); (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            revealed_tiles: vec![false; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            blocked_tiles: vec![false; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)],
            bloodstains: HashSet::new(),
            rooms: Vec::new(),
            width: MAX_WIDTH,
            height: MAX_HEIGHT,
        };
    
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
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                info!("Spawned new room with position ({}, {}) - ({}, {})", new_room.x1, new_room.y1, new_room.x2, new_room.y2);
                map.rooms.push(new_room);            
            }
        }

        let (stair_x, stair_y) = map.rooms[map.rooms.len() - 1].center();
        map.tiles[xy_idx(stair_x, stair_y)] = TileType::DownStairs;

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
