use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

#[derive(Resource)]
pub struct CurrentLevel {
    pub level_number: usize,
    pub waves_remaining: Vec<WaveConfig>,
}

pub struct WaveConfig {
    pub trigger: WaveTrigger,
    pub patterns: Vec<PatternPlacement>,
}

pub struct PatternPlacement {
    pub cells: Vec<(i32, i32)>,
    pub offset: (i32, i32),
}

pub enum WaveTrigger {
    CellCountBelow(usize),
}

// Classic Game of Life patterns
pub fn glider() -> Vec<(i32, i32)> {
    vec![(0, 0), (1, 0), (2, 0), (2, 1), (1, 2)]
}

pub fn blinker() -> Vec<(i32, i32)> {
    vec![(0, 0), (1, 0), (2, 0)]
}

pub fn toad() -> Vec<(i32, i32)> {
    vec![(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1)]
}

pub fn beacon() -> Vec<(i32, i32)> {
    vec![(0, 0), (1, 0), (0, 1), (3, 2), (2, 3), (3, 3)]
}

pub fn r_pentomino() -> Vec<(i32, i32)> {
    vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]
}

pub fn lwss() -> Vec<(i32, i32)> {
    vec![
        (0, 0), (3, 0),
        (4, 1),
        (0, 2), (4, 2),
        (1, 3), (2, 3), (3, 3), (4, 3),
    ]
}

pub fn pulsar() -> Vec<(i32, i32)> {
    vec![
        (2, 0), (3, 0), (4, 0), (8, 0), (9, 0), (10, 0),
        (0, 2), (5, 2), (7, 2), (12, 2),
        (0, 3), (5, 3), (7, 3), (12, 3),
        (0, 4), (5, 4), (7, 4), (12, 4),
        (2, 5), (3, 5), (4, 5), (8, 5), (9, 5), (10, 5),
        (2, 7), (3, 7), (4, 7), (8, 7), (9, 7), (10, 7),
        (0, 8), (5, 8), (7, 8), (12, 8),
        (0, 9), (5, 9), (7, 9), (12, 9),
        (0, 10), (5, 10), (7, 10), (12, 10),
        (2, 12), (3, 12), (4, 12), (8, 12), (9, 12), (10, 12),
    ]
}

/// Add `count` random cells adjacent to existing cells in the pattern.
/// Each mutation touches the pattern (shares a neighbor edge).
fn mutate_pattern(cells: &mut Vec<(i32, i32)>, count: usize, rng: &mut impl Rng) {
    let mut existing: HashSet<(i32, i32)> = cells.iter().copied().collect();

    for _ in 0..count {
        // Collect all empty neighbor positions adjacent to existing cells
        let mut candidates: Vec<(i32, i32)> = Vec::new();
        for &(x, y) in existing.iter() {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let pos = (x + dx, y + dy);
                    if !existing.contains(&pos) {
                        candidates.push(pos);
                    }
                }
            }
        }
        candidates.sort();
        candidates.dedup();

        if candidates.is_empty() {
            break;
        }

        let pick = candidates[rng.gen_range(0..candidates.len())];
        existing.insert(pick);
        cells.push(pick);
    }
}

/// Pick a random base pattern appropriate for the level.
fn pick_pattern(rng: &mut impl Rng, level: usize) -> Vec<(i32, i32)> {
    // No blocks — they're boring 2x2 still lifes
    if level <= 3 {
        match rng.gen_range(0..3) {
            0 => blinker(),
            1 => toad(),
            _ => glider(),
        }
    } else if level <= 6 {
        match rng.gen_range(0..4) {
            0 => blinker(),
            1 => toad(),
            2 => beacon(),
            _ => glider(),
        }
    } else if level <= 10 {
        match rng.gen_range(0..5) {
            0 => toad(),
            1 => glider(),
            2 => beacon(),
            3 => r_pentomino(),
            _ => lwss(),
        }
    } else {
        match rng.gen_range(0..6) {
            0 => glider(),
            1 => toad(),
            2 => beacon(),
            3 => r_pentomino(),
            4 => lwss(),
            _ => pulsar(),
        }
    }
}

pub fn get_level(level_number: usize, rng: &mut impl Rng) -> (Vec<PatternPlacement>, Vec<WaveConfig>, f32) {
    // Enemy count: level 1 = 4, level 2 = 5, etc.
    let enemy_count = 3 + level_number;

    // Mutated enemy count: ceil(level / 2)
    // Level 1 = 1, level 2 = 1, level 3 = 2, level 4 = 2, level 5 = 3...
    let mutated_count = (level_number + 1) / 2;

    // Place enemies in a ring around center, all at safe distance
    let min_dist: f32 = 12.0;
    let max_dist: f32 = 25.0;
    let mut patterns = Vec::new();

    for i in 0..enemy_count {
        let angle = (i as f32 / enemy_count as f32) * std::f32::consts::TAU;
        // Vary distance so they don't form a perfect circle
        let dist = min_dist + rng.gen_range(0.0..=(max_dist - min_dist));
        let ox = (angle.cos() * dist) as i32;
        let oy = (angle.sin() * dist) as i32;

        let mut cells = pick_pattern(rng, level_number);

        // Mutate this enemy if it's among the first `mutated_count`
        if i < mutated_count {
            // Mutation grows with level: 1 cell at level 1-2, 2 at level 3-4, etc.
            let mutation_cells = (level_number + 1) / 2;
            mutate_pattern(&mut cells, mutation_cells, rng);
        }

        patterns.push(PatternPlacement { cells, offset: (ox, oy) });
    }

    // Wave reinforcements: one wave per 3 levels, triggered when few cells remain
    let waves = if level_number >= 3 && level_number % 3 == 0 {
        let wave_enemy_count = 1 + level_number / 4;
        let mut wave_patterns = Vec::new();
        for j in 0..wave_enemy_count {
            let angle = (j as f32 / wave_enemy_count as f32) * std::f32::consts::TAU
                + std::f32::consts::FRAC_PI_4; // offset from initial placement
            let dist = 20.0 + rng.gen_range(0.0..5.0);
            let ox = (angle.cos() * dist) as i32;
            let oy = (angle.sin() * dist) as i32;
            let cells = pick_pattern(rng, level_number);
            wave_patterns.push(PatternPlacement { cells, offset: (ox, oy) });
        }
        vec![WaveConfig {
            trigger: WaveTrigger::CellCountBelow(3 + level_number),
            patterns: wave_patterns,
        }]
    } else {
        vec![]
    };

    // GoL tick rate: starts slow, gets faster. Floors at 0.1s
    let tick_rate = (0.45 - level_number as f32 * 0.03).max(0.1);

    (patterns, waves, tick_rate)
}
