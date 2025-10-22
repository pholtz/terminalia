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

    let max_width: i32 = MAX_WIDTH.try_into().unwrap();
    let max_height: i32 = MAX_HEIGHT.try_into().unwrap();

    // Make the boundaries walls
    for x in 0..max_width {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, max_height - 1)] = TileType::Wall;
    }
    for y in 0..max_height {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(max_height - 1, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..(max_width * max_height) {
        let x = rng.roll_dice(1, max_width - 1);
        let y = rng.roll_dice(1, max_height - 1);
        let idx = xy_idx(x, y);
        if idx != xy_idx(max_width / 2, max_height / 2) {
            map[idx] = TileType::Wall;
        }
    }

    map
}
