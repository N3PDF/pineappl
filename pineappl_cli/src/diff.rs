use lhapdf::Pdf;
use pineappl::grid::Grid;
use prettytable::{cell, Row, Table};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use super::helpers::create_table;

pub fn subcommand(
    input1: &str,
    input2: &str,
    pdfset: &str,
) -> Result<Table, Box<dyn Error>> {
    let grid1 = Grid::read(BufReader::new(File::open(input1)?))?;
    let grid2 = Grid::read(BufReader::new(File::open(input2)?))?;
    let pdf = pdfset
        .parse()
        .map_or_else(|_| Pdf::with_setname_and_member(pdfset, 0), Pdf::with_lhaid);

    let mut table = create_table();

    if grid1.bin_limits() == grid2.bin_limits() {
        let orders1: HashSet<_> = grid1
            .orders()
            .iter()
            .filter(|order| (order.logxir == 0) && (order.logxif == 0))
            .collect();
        let orders2: HashSet<_> = grid2
            .orders()
            .iter()
            .filter(|order| (order.logxir == 0) && (order.logxif == 0))
            .collect();

        let mut diff1: Vec<_> = orders1.difference(&orders2).collect();
        diff1.sort();
        let diff1 = diff1;
        let mut diff2: Vec<_> = orders2.difference(&orders1).collect();
        diff2.sort();
        let diff2 = diff2;

        if !diff1.is_empty() || !diff2.is_empty() {
            print!("--- Orders: ");
            for order in diff1.iter() {
                if order.alphas == 0 {
                    print!("O(a^{}) ", order.alpha);
                } else if order.alpha == 0 {
                    print!("O(as^{}) ", order.alphas);
                } else {
                    print!("O(as^{} a^{}) ", order.alphas, order.alpha);
                }
            }
            println!();
            print!("+++ Orders: ");
            for order in diff2.iter() {
                if order.alphas == 0 {
                    print!("O(a^{}) ", order.alpha);
                } else if order.alpha == 0 {
                    print!("O(as^{}) ", order.alphas);
                } else {
                    print!("O(as^{} a^{}) ", order.alphas, order.alpha);
                }
            }
            println!();
            println!();
        }

        let mut title = Row::empty();
        title.add_cell(cell!(c->"bin"));
        title.add_cell(cell!(c->"xmin"));
        title.add_cell(cell!(c->"xmax"));

        let mut orders: Vec<_> = orders1.intersection(&orders2).collect();
        orders.sort();
        let orders = orders;

        for order in orders.iter() {
            let mut cell = cell!(c->&format!("O(as^{} a^{})", order.alphas, order.alpha));
            cell.set_hspan(3);
            title.add_cell(cell);
        }

        table.set_titles(title);

        let order_results1: Vec<Vec<f64>> = orders
            .iter()
            .map(|order| {
                let mut order_mask = vec![false; grid1.orders().len()];
                order_mask[grid1.orders().iter().position(|o| o == **order).unwrap()] = true;
                grid1.convolute(
                    &|id, x1, q2| pdf.xfx_q2(id, x1, q2),
                    &|id, x2, q2| pdf.xfx_q2(id, x2, q2),
                    &|q2| pdf.alphas_q2(q2),
                    &order_mask,
                    &[],
                    &[],
                    &[(1.0, 1.0)],
                )
            })
            .collect();
        let order_results2: Vec<Vec<f64>> = orders
            .iter()
            .map(|order| {
                let mut order_mask = vec![false; grid2.orders().len()];
                order_mask[grid2.orders().iter().position(|o| o == **order).unwrap()] = true;
                grid2.convolute(
                    &|id, x1, q2| pdf.xfx_q2(id, x1, q2),
                    &|id, x2, q2| pdf.xfx_q2(id, x2, q2),
                    &|q2| pdf.alphas_q2(q2),
                    &order_mask,
                    &[],
                    &[],
                    &[(1.0, 1.0)],
                )
            })
            .collect();

        for (bin, limits) in grid1.bin_limits().limits().windows(2).enumerate() {
            let row = table.add_empty_row();

            row.add_cell(cell!(r->bin));
            row.add_cell(cell!(r->limits[0]));
            row.add_cell(cell!(r->limits[1]));

            for (result1, result2) in order_results1.iter().zip(order_results2.iter()) {
                let result1 = result1[bin];
                let result2 = result2[bin];
                row.add_cell(cell!(r->&format!("{:.7e}", result1)));
                row.add_cell(cell!(r->&format!("{:.7e}", result2)));
                row.add_cell(cell!(r->&format!("{:.3e}",
                    if result1 == result2 { 0.0 } else { result1 / result2 - 1.0 })));
            }
        }
    } else {
        print!("--- Bin limits: ");
        for limit in grid1.bin_limits().limits() {
            print!("{} ", limit);
        }
        println!();
        print!("+++ Bin limits: ");
        for limit in grid2.bin_limits().limits() {
            print!("{} ", limit);
        }
        println!();
    }

    Ok(table)
}
