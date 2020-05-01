//! Provides an implementation of the `Grid` trait with n-tuples.

use super::grid::{Subgrid, SubgridData};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct Ntuple {
    x1: f64,
    x2: f64,
    q2: f64,
    weight: f64,
}

impl Ntuple {
    const fn new(x1: f64, x2: f64, q2: f64, weight: f64) -> Self {
        Self { x1, x2, q2, weight }
    }
}

/// Structure holding a grid with an n-tuple as the storage method for weights.
#[derive(Deserialize, Serialize)]
pub struct NtupleSubgrid {
    ntuples: Vec<Vec<Vec<Ntuple>>>,
    subgrid_data: SubgridData,
}

impl NtupleSubgrid {
    /// Constructor.
    #[must_use]
    pub fn new(lumi_len: usize, bins: usize, subgrid_data: SubgridData) -> Self {
        assert!(lumi_len > 0);
        assert!(bins > 0);

        Self {
            ntuples: vec![vec![vec![]; lumi_len]; bins],
            subgrid_data,
        }
    }
}

#[typetag::serde]
impl Subgrid for NtupleSubgrid {
    fn fill(&mut self, x1: f64, x2: f64, q2: f64, obs_index: usize, weights: &[f64]) {
        assert!(weights.len() == self.ntuples[0].len());

        for (lumi_index, &weight) in weights.iter().enumerate() {
            self.ntuples[obs_index][lumi_index].push(Ntuple::new(x1, x2, q2, weight));
        }
    }

    fn scale(&mut self, factor: f64) {
        for i in &mut self.ntuples {
            for j in i.iter_mut() {
                for k in j.iter_mut() {
                    k.weight *= factor;
                }
            }
        }
    }

    fn data(&self) -> &SubgridData {
        &self.subgrid_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntuple_subgrid_fill() {
        // a single subgrid
        let subgrid_data = SubgridData {
            alphas: 0,
            alpha: 2,
            logxir: 0,
            logxif: 0,
        };

        let mut grid = NtupleSubgrid::new(1, 4, subgrid_data);

        // check first bin
        grid.fill(0.5, 0.75, 1.0, 0, &[2.0]);
        assert_eq!(grid.ntuples[0][0][0], Ntuple::new(0.5, 0.75, 1.0, 2.0));

        // check second bin
        grid.fill(0.75, 0.5, 2.0, 1, &[3.0]);
        assert_eq!(grid.ntuples[0][0][0], Ntuple::new(0.5, 0.75, 1.0, 2.0));
        assert_eq!(grid.ntuples[1][0][0], Ntuple::new(0.75, 0.5, 2.0, 3.0));

        // check third bin
        grid.fill(0.125, 0.25, 3.0, 2, &[4.0]);
        assert_eq!(grid.ntuples[0][0][0], Ntuple::new(0.5, 0.75, 1.0, 2.0));
        assert_eq!(grid.ntuples[1][0][0], Ntuple::new(0.75, 0.5, 2.0, 3.0));
        assert_eq!(grid.ntuples[2][0][0], Ntuple::new(0.125, 0.25, 3.0, 4.0));

        // check fourth bin
        grid.fill(0.5, 0.5, 4.0, 3, &[5.0]);
        assert_eq!(grid.ntuples[0][0][0], Ntuple::new(0.5, 0.75, 1.0, 2.0));
        assert_eq!(grid.ntuples[1][0][0], Ntuple::new(0.75, 0.5, 2.0, 3.0));
        assert_eq!(grid.ntuples[2][0][0], Ntuple::new(0.125, 0.25, 3.0, 4.0));
        assert_eq!(grid.ntuples[3][0][0], Ntuple::new(0.5, 0.5, 4.0, 5.0));
    }
}
