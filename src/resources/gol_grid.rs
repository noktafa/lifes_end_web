use bevy::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct LifeGrid {
    pub alive_cells: HashSet<(i32, i32)>,
    pub tick_timer: Timer,
    pub bounds: (i32, i32), // half-width, half-height in grid coords
    cell_age: HashMap<(i32, i32), u16>,
}

impl LifeGrid {
    pub fn new(tick_rate: f32, bounds: (i32, i32)) -> Self {
        Self {
            alive_cells: HashSet::new(),
            tick_timer: Timer::from_seconds(tick_rate, TimerMode::Repeating),
            bounds,
            cell_age: HashMap::new(),
        }
    }

    pub fn tick(&mut self, rng: &mut impl Rng) {
        let mut neighbor_counts: HashMap<(i32, i32), u8> = HashMap::new();

        for &(x, y) in &self.alive_cells {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    *neighbor_counts.entry((x + dx, y + dy)).or_insert(0) += 1;
                }
            }
        }

        let (bw, bh) = self.bounds;
        let mut next_gen = HashSet::new();
        for (&pos, &count) in &neighbor_counts {
            if pos.0.abs() > bw || pos.1.abs() > bh {
                continue;
            }
            if count == 3 || (count == 2 && self.alive_cells.contains(&pos)) {
                next_gen.insert(pos);
            }
        }

        // Update cell ages: survivors get +1, new cells start at 1, dead cells removed
        let mut new_ages: HashMap<(i32, i32), u16> = HashMap::new();
        for &pos in &next_gen {
            let age = self.cell_age.get(&pos).copied().unwrap_or(0) + 1;
            new_ages.insert(pos, age);
        }
        self.cell_age = new_ages;

        // Stagnation mutation: cells alive for 16+ ticks spawn a random neighbor
        let stale: Vec<(i32, i32)> = self
            .cell_age
            .iter()
            .filter(|(&_, &age)| age >= 16)
            .map(|(&pos, _)| pos)
            .collect();

        for pos in stale {
            // Find empty adjacent positions
            let mut candidates: Vec<(i32, i32)> = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let n = (pos.0 + dx, pos.1 + dy);
                    if n.0.abs() <= bw && n.1.abs() <= bh && !next_gen.contains(&n) {
                        candidates.push(n);
                    }
                }
            }
            if !candidates.is_empty() {
                let pick = candidates[rng.gen_range(0..candidates.len())];
                next_gen.insert(pick);
                self.cell_age.insert(pick, 0);
            }
            // Reset the stale cell's age so it doesn't mutate every tick
            self.cell_age.insert(pos, 0);
        }

        self.alive_cells = next_gen;
    }

    pub fn add_pattern(&mut self, cells: &[(i32, i32)], offset: (i32, i32)) {
        for &(x, y) in cells {
            self.alive_cells.insert((x + offset.0, y + offset.1));
        }
    }

    pub fn clear_radius(&mut self, center: (i32, i32), radius: i32) {
        self.alive_cells.retain(|&(x, y)| {
            let dx = x - center.0;
            let dy = y - center.1;
            dx * dx + dy * dy > radius * radius
        });
    }

    /// Find connected groups of cells (8-connectivity flood fill).
    pub fn find_groups(&self) -> Vec<Vec<(i32, i32)>> {
        let mut visited: HashSet<(i32, i32)> = HashSet::new();
        let mut groups: Vec<Vec<(i32, i32)>> = Vec::new();

        for &cell in &self.alive_cells {
            if visited.contains(&cell) {
                continue;
            }
            // BFS flood fill
            let mut group = Vec::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(cell);
            visited.insert(cell);

            while let Some(pos) = queue.pop_front() {
                group.push(pos);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor = (pos.0 + dx, pos.1 + dy);
                        if self.alive_cells.contains(&neighbor) && !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            groups.push(group);
        }
        groups
    }

    /// Nuke: for each enemy group, delete 3 random cells then add 5 adjacent to remaining.
    pub fn nuke(&mut self, rng: &mut impl Rng) {
        let (bw, bh) = self.bounds;
        let groups = self.find_groups();

        for mut group in groups {
            // Delete up to 3 random cells from this group
            let delete_count = group.len().min(3);
            for _ in 0..delete_count {
                if group.is_empty() {
                    break;
                }
                let idx = rng.gen_range(0..group.len());
                let removed = group.swap_remove(idx);
                self.alive_cells.remove(&removed);
                self.cell_age.remove(&removed);
            }

            // Add 5 random adjacent cells to the remaining group members
            if group.is_empty() {
                continue;
            }
            let group_set: HashSet<(i32, i32)> = group.iter().copied().collect();
            let mut added = 0;
            let mut attempts = 0;
            while added < 5 && attempts < 50 {
                attempts += 1;
                // Pick a random cell from the group
                let anchor = group[rng.gen_range(0..group.len())];
                // Pick a random neighbor direction
                let dx = rng.gen_range(-1..=1_i32);
                let dy = rng.gen_range(-1..=1_i32);
                if dx == 0 && dy == 0 {
                    continue;
                }
                let new_pos = (anchor.0 + dx, anchor.1 + dy);
                // Must be in bounds, not already alive, not in the original group
                if new_pos.0.abs() > bw || new_pos.1.abs() > bh {
                    continue;
                }
                if self.alive_cells.contains(&new_pos) || group_set.contains(&new_pos) {
                    continue;
                }
                self.alive_cells.insert(new_pos);
                self.cell_age.insert(new_pos, 0);
                added += 1;
            }
        }
    }
}
