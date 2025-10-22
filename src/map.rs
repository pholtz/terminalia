pub const MAX_WIDTH: i32 = 80;
pub const MAX_HEIGHT: i32 = 50;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * MAX_WIDTH as usize) + x as usize
}

pub fn idx_xy(idx: usize) -> (i32, i32) {
    let x = (idx % (MAX_WIDTH as usize)) as i32;
    let y = (idx / (MAX_WIDTH as usize)) as i32;
    (x, y)
}

pub fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (MAX_WIDTH as usize) * (MAX_HEIGHT as usize)];

    // Make the boundaries walls
    for x in 0..MAX_WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, MAX_HEIGHT - 1)] = TileType::Wall;
    }
    for y in 0..MAX_HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(MAX_WIDTH - 1, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..25 {
        let x = rng.roll_dice(1, MAX_WIDTH - 1);
        let y = rng.roll_dice(1, MAX_HEIGHT - 1);
        let idx = xy_idx(x, y);
        if idx != xy_idx(MAX_WIDTH / 2, MAX_HEIGHT / 2) {
            map[idx] = TileType::Wall;
        }
    }

    map
}
