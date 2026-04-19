use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn dx(&self) -> i32 { match self { Direction::West => -1, Direction::East => 1, Direction::North => 0, Direction::South => 0 } }
    pub fn dz(&self) -> i32 { match self { Direction::West => 0, Direction::East => 0, Direction::North => -1, Direction::South => 1 } }
    pub fn left(&self) -> Self { match self { Direction::North => Direction::West, Direction::West => Direction::South, Direction::South => Direction::East, Direction::East => Direction::North } }
    pub fn right(&self) -> Self { match self { Direction::North => Direction::East, Direction::East => Direction::South, Direction::South => Direction::West, Direction::West => Direction::North } }
    pub fn opposite(&self) -> Self { match self { Direction::North => Direction::South, Direction::South => Direction::North, Direction::East => Direction::West, Direction::West => Direction::East } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Empty,
    Flat,
    Wall,
    Slope(Direction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tile_type: TileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: u32,
    pub tiles: Vec<Tile>,
}

impl Room {
    pub fn generate_room(seed: u16) -> Self {
        let mut tiles = Vec::new();
        let width: usize = 32;
        let depth: usize = 32;

        struct Lcg { state: u32 }
        impl Lcg {
            fn new(s: u16) -> Self { Self { state: s as u32 + 1 } }
            fn next(&mut self) -> u32 {
                self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
                self.state
            }
            fn rng(&mut self, n: usize) -> usize { (self.next() as usize) % n }
        }
        let mut rng = Lcg::new(seed);

        // 4 яруса, высоты 0/2/4/6, по 8 тайлов по X
        // Нечётные высоты (1/3/5) = slope-ячейки
        let num_terraces = 4usize;
        let terrace_w = width / num_terraces; // 8
        let terrace_h: [i32; 4] = [0, 2, 4, 6];

        let mut hmap = vec![vec![0i32; depth]; width];
        for x in 0..width {
            let t = (x / terrace_w).min(num_terraces - 1);
            for z in 0..depth { hmap[x][z] = terrace_h[t]; }
        }

        // Рампы на каждой границе
        for boundary in 1..num_terraces {
            let x_b  = boundary * terrace_w;
            let h_lo = terrace_h[boundary - 1];
            let h_mid = h_lo + 1; // нечётная высота-рампа

            // ── Фронтальные рампы (Slope West) — 1-2 штуки шириной 2 тайла ──
            let n_front = 1 + rng.rng(2);
            let zone = (depth / 2) / n_front;
            for i in 0..n_front {
                let z0 = (depth / 4 + i * zone + rng.rng(zone.max(1))).min(depth - 2);
                hmap[x_b][z0]     = h_mid;
                hmap[x_b][z0 + 1] = h_mid;
            }

            // ── Боковая рампа (Slope South или North) ─────────────────────────
            // Схема (South): опускаем угол x_b до h_lo, slope-ячейка перед ним:
            //   hmap[x_b][depth-1] = h_lo  → угол доступен с нижнего яруса
            //   hmap[x_b][depth-2] = h_mid → ds=1 (h_lo), h_n=h_hi >= h_mid → Slope(South)
            // Игрок идёт вдоль нижнего яруса к z=depth-1, заходит в угол,
            // поворачивает на север и заезжает на рампу.
            if rng.rng(2) == 0 {
                hmap[x_b][depth - 1] = h_lo;
                hmap[x_b][depth - 2] = h_mid;
            } else {
                hmap[x_b][0] = h_lo;
                hmap[x_b][1] = h_mid;
            }
        }

        // ── Генерация тайлов: только поверхность + видимые стены ─────────────
        for x in 0..width {
            for z in 0..depth {
                let xi = x as i32;
                let zi = z as i32;
                let h  = hmap[x][z];

                let h_w = if x > 0       { hmap[x-1][z] } else { h };
                let h_e = if x+1 < width { hmap[x+1][z] } else { h };
                let h_n = if z > 0       { hmap[x][z-1] } else { h };
                let h_s = if z+1 < depth { hmap[x][z+1] } else { h };

                // Slope только у нечётных (рамп-ячейки)
                let top_type = if h % 2 == 1 {
                    if      h - h_w == 1 && h_e >= h { TileType::Slope(Direction::West)  }
                    else if h - h_e == 1 && h_w >= h { TileType::Slope(Direction::East)  }
                    else if h - h_n == 1 && h_s >= h { TileType::Slope(Direction::North) }
                    else if h - h_s == 1 && h_n >= h { TileType::Slope(Direction::South) }
                    else { TileType::Flat }
                } else {
                    TileType::Flat
                };

                tiles.push(Tile { x: xi, y: h, z: zi, tile_type: top_type });

                // Стены (все ячейки, включая slope — wedge закрывает вход визуально)
                let min_n = *[h_w, h_e, h_n, h_s].iter().min().unwrap();
                for y in min_n..h {
                    tiles.push(Tile { x: xi, y, z: zi, tile_type: TileType::Wall });
                }
            }
        }

        Self { id: 1, tiles }
    }
}
