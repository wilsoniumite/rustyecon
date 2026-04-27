use crate::state::{GameData, SimState};
use crate::types::ids::{GoodId, MarketNodeId};
use std::io::{self, Write};
use std::path::Path;

struct PriceRow {
    tick: u64,
    node_id: u32,
    good_id: u32,
    price: f64,
    imbalance: f64,
}

struct InventoryRow {
    tick: u64,
    entity_type: &'static str,
    entity_id: u32,
    good_id: u32,
    qty: f64,
}

/// Collects per-tick market and inventory snapshots during a run.
/// Call `record` after each tick, then `write_csv` at the end.
pub struct Telemetry {
    prices: Vec<PriceRow>,
    inventories: Vec<InventoryRow>,
    num_nodes: usize,
    num_goods: usize,
}

impl Telemetry {
    pub fn new(num_nodes: usize, num_goods: usize) -> Self {
        Self {
            prices: Vec::new(),
            inventories: Vec::new(),
            num_nodes,
            num_goods,
        }
    }

    pub fn record(&mut self, state: &SimState) {
        let tick = state.tick;

        for node_idx in 0..self.num_nodes {
            let node = MarketNodeId(node_idx as u32);
            for good_idx in 0..self.num_goods {
                let good = GoodId(good_idx as u32);
                self.prices.push(PriceRow {
                    tick,
                    node_id: node_idx as u32,
                    good_id: good_idx as u32,
                    price: state.price(node, good),
                    imbalance: state.imbalance(node, good),
                });
            }
        }

        for b in &state.buildings {
            for &(good, qty) in b.inventory.goods() {
                if qty > 0.0 {
                    self.inventories.push(InventoryRow {
                        tick,
                        entity_type: "building",
                        entity_id: b.id.0,
                        good_id: good.0,
                        qty,
                    });
                }
            }
        }

        for p in &state.pop_groups {
            for &(good, qty) in p.inventory.goods() {
                if qty > 0.0 {
                    self.inventories.push(InventoryRow {
                        tick,
                        entity_type: "pop",
                        entity_id: p.id.0,
                        good_id: good.0,
                        qty,
                    });
                }
            }
        }
    }

    pub fn write_csv(&self, dir: &Path, _game_data: &GameData) -> io::Result<()> {
        std::fs::create_dir_all(dir)?;

        {
            let mut f = std::fs::File::create(dir.join("prices.csv"))?;
            writeln!(f, "tick,node_id,good_id,price,imbalance")?;
            for r in &self.prices {
                writeln!(
                    f,
                    "{},{},{},{:.6},{:.6}",
                    r.tick, r.node_id, r.good_id, r.price, r.imbalance
                )?;
            }
        }

        {
            let mut f = std::fs::File::create(dir.join("inventories.csv"))?;
            writeln!(f, "tick,entity_type,entity_id,good_id,qty")?;
            for r in &self.inventories {
                writeln!(
                    f,
                    "{},{},{},{},{:.6}",
                    r.tick, r.entity_type, r.entity_id, r.good_id, r.qty
                )?;
            }
        }

        self.write_inflation_csv(dir)?;

        Ok(())
    }

    fn write_inflation_csv(&self, dir: &Path) -> io::Result<()> {
        use std::collections::HashMap;

        let mut baseline: HashMap<(u32, u32), f64> = HashMap::new();
        if let Some(first_tick) = self.prices.first().map(|r| r.tick) {
            for r in self.prices.iter().filter(|r| r.tick == first_tick) {
                baseline.insert((r.node_id, r.good_id), r.price);
            }
        }
        if baseline.is_empty() {
            return Ok(());
        }

        let mut by_tick: std::collections::BTreeMap<u64, Vec<&PriceRow>> = Default::default();
        for r in &self.prices {
            by_tick.entry(r.tick).or_default().push(r);
        }

        let mut index_by_tick: HashMap<u64, f64> = HashMap::new();
        let mut rows: Vec<(u64, f64, String)> = Vec::new();

        for (&tick, tick_rows) in &by_tick {
            let ratios: Vec<f64> = tick_rows.iter()
                .filter_map(|r| {
                    baseline.get(&(r.node_id, r.good_id)).map(|&base| {
                        if base > 1e-12 { r.price / base } else { 1.0 }
                    })
                })
                .collect();
            if ratios.is_empty() { continue; }
            let index = ratios.iter().sum::<f64>() / ratios.len() as f64;
            index_by_tick.insert(tick, index);
            rows.push((tick, index, String::new()));
        }

        for (tick, index, yoy) in &mut rows {
            if *tick >= 52 {
                if let Some(&prior) = index_by_tick.get(&(*tick - 52)) {
                    if prior > 1e-12 {
                        *yoy = format!("{:.4}", (*index / prior - 1.0) * 100.0);
                        continue;
                    }
                }
            }
            *yoy = String::new();
        }

        let mut f = std::fs::File::create(dir.join("inflation.csv"))?;
        writeln!(f, "tick,price_index,yoy_inflation_pct")?;
        for (tick, index, yoy) in &rows {
            writeln!(f, "{},{:.6},{}", tick, index, yoy)?;
        }
        Ok(())
    }
}
