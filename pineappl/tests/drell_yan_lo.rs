use float_cmp::approx_eq;
use lhapdf::Pdf;
use pineappl::bin::BinRemapper;
use pineappl::grid::{Grid, Ntuple, Order};
use pineappl::lumi_entry;
use pineappl::subgrid::{ExtraSubgridParams, SubgridParams};
use rand::Rng;
use rand_pcg::Pcg64;
use std::f64::consts::PI;
use std::io::Cursor;
use std::mem;

// If equation numbers are given, they are from Max Huber's PhD thesis:
//   'Radiative corrections to the neutral-current Drell-Yan process'

// Eq. (2.9) - gamma-gamma contribution to DY pair production
fn int_photo(s: f64, t: f64, u: f64) -> f64 {
    let alpha0: f64 = 1.0 / 137.03599911;
    alpha0.powi(2) / 2.0 / s * (t / u + u / t)
}

struct Psp2to2 {
    s: f64,
    t: f64,
    u: f64,
    x1: f64,
    x2: f64,
    jacobian: f64,
}

fn hadronic_pspgen(rng: &mut impl Rng, mmin: f64, mmax: f64) -> Psp2to2 {
    let smin = mmin * mmin;
    let smax = mmax * mmax;

    let mut jacobian = 1.0;

    let r1 = rng.gen::<f64>();
    let r2 = rng.gen::<f64>();
    let tau0 = smin / smax;
    let tau = tau0.powf(r1);
    let y = tau.powf(1.0 - r2);
    let x1 = y;
    let x2 = tau / y;
    let s = tau * smax;
    jacobian *= tau * tau0.ln().powi(2) * r1;

    // theta integration (in the CMS)
    let cos_theta = 2.0 * rng.gen::<f64>() - 1.0;
    jacobian *= 2.0;

    let t = -0.5 * s * (1.0 - cos_theta);
    let u = -0.5 * s * (1.0 + cos_theta);

    // phi integration
    jacobian *= 2.0 * PI;

    Psp2to2 {
        s,
        t,
        u,
        x1,
        x2,
        jacobian,
    }
}

fn fill_drell_yan_lo_grid(
    rng: &mut impl Rng,
    calls: usize,
    subgrid_type: &str,
    dynamic: bool,
) -> anyhow::Result<Grid> {
    let lumi = vec![
        // photons
        lumi_entry![22, 22, 1.0],
    ];

    // only LO alpha^2
    let orders = vec![Order {
        alphas: 0,
        alpha: 2,
        logxir: 0,
        logxif: 0,
    }];

    // we bin in rapidity from 0 to 2.4 in steps of 0.1
    let bin_limits: Vec<f64> = (0..=24).map(|x| x as f64 / 10.0).collect();

    let mut subgrid_params = SubgridParams::default();
    let mut extra = ExtraSubgridParams::default();

    subgrid_params.set_q2_bins(30);
    subgrid_params.set_q2_max(1e6);
    subgrid_params.set_q2_min(1e2);
    subgrid_params.set_q2_order(3);
    subgrid_params.set_reweight(true);
    subgrid_params.set_x_bins(50);
    subgrid_params.set_x_max(1.0);
    subgrid_params.set_x_min(2e-7);
    subgrid_params.set_x_order(3);
    extra.set_x2_bins(50);
    extra.set_x2_max(1.0);
    extra.set_x2_min(2e-7);
    extra.set_x2_order(3);

    // create the PineAPPL grid
    let mut grid = Grid::with_subgrid_type(
        lumi,
        orders,
        bin_limits,
        subgrid_params,
        extra,
        subgrid_type,
    )?;

    // in GeV^2 pbarn
    let hbarc2 = 3.893793721e8;

    for _ in 0..calls {
        // generate a phase-space point
        let Psp2to2 {
            s,
            t,
            u,
            x1,
            x2,
            mut jacobian,
        } = hadronic_pspgen(rng, 10.0, 7000.0);

        let ptl = (t * u / s).sqrt();
        let mll = s.sqrt();
        let yll = 0.5 * (x1 / x2).ln();
        let ylp = (yll + (0.5 * mll / ptl).acosh()).abs();
        let ylm = (yll - (0.5 * mll / ptl).acosh()).abs();

        jacobian *= hbarc2 / (calls as f64);

        // cuts for LO for the invariant-mass slice containing the Z-peak from CMSDY2D11
        if (ptl < 14.0)
            || (yll.abs() > 2.4)
            || (ylp > 2.4)
            || (ylm > 2.4)
            || (mll < 60.0)
            || (mll > 120.0)
        {
            continue;
        }

        let weight = jacobian * int_photo(s, u, t);
        let q2 = if dynamic { mll * mll } else { 90.0 * 90.0 };

        grid.fill(0, yll.abs(), 0, &Ntuple { x1, x2, q2, weight });
    }

    Ok(grid)
}

#[test]
fn dy_aa_lagrange_subgrid_static() -> anyhow::Result<()> {
    // suppress LHAPDF banners
    lhapdf::set_verbosity(0);

    let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96);
    let mut grid = fill_drell_yan_lo_grid(&mut rng, 500_000, "LagrangeSubgrid", false)?;

    grid.merge(fill_drell_yan_lo_grid(
        &mut rng,
        500_000,
        "LagrangeSubgrid",
        false,
    )?)?;
    grid.scale(0.5);

    let pdf_set = "NNPDF31_nlo_as_0118_luxqed";

    assert!(lhapdf::available_pdf_sets().iter().any(|x| x == &pdf_set));

    let pdf = Pdf::with_setname_and_member(&pdf_set, 0);
    let xfx = |id, x, q2| pdf.xfx_q2(id, x, q2);
    let alphas = |_| 0.0;

    // check `read` and `write`
    let mut file = Cursor::new(Vec::new());
    grid.write(&mut file)?;
    file.set_position(0);
    mem::drop(grid);
    let mut grid = Grid::read(&mut file)?;

    // some useless scalings
    grid.scale_by_order(10.0, 0.5, 10.0, 10.0, 1.0);
    grid.scale_by_order(10.0, 1.0, 10.0, 10.0, 4.0);

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);
    let reference = vec![
        5.29438499470369e-1,
        5.407794857747981e-1,
        5.696902048421408e-1,
        5.028293125183927e-1,
        4.91700010813869e-1,
        4.946801648085449e-1,
        4.9188982902741324e-1,
        4.486342876707396e-1,
        4.5078095484478575e-1,
        4.106209790738961e-1,
        3.602582665914635e-1,
        3.275794060602094e-1,
        2.7928887723972295e-1,
        2.498545969916396e-1,
        2.108027399225483e-1,
        1.7799404895027734e-1,
        1.5411875388722898e-1,
        1.1957877908479132e-1,
        9.39935306988927e-2,
        6.719034949888109e-2,
        5.136619446064035e-2,
        3.5716156871884834e-2,
        2.067251421406746e-2,
        7.300411258011377e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    // optimize the grid
    grid.optimize();

    // make a two-dimensional distribution out of it
    grid.set_remapper(BinRemapper::new(
        vec![0.1; 24],
        (0..24)
            .flat_map(|index| {
                let index = f64::from(index);
                vec![(60.0, 120.0), (index * 0.1, (index + 1.0) * 0.1)]
            })
            .collect::<Vec<(f64, f64)>>(),
    )?)?;

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);

    // results are slightly different because of the static scale detection - the interpolation
    // error in the Q^2 dimension is removed
    let reference = vec![
        5.2943850298637385e-1,
        5.4077949153675209e-1,
        5.6969021381443985e-1,
        5.0282932188764318e-1,
        4.9170002525140877e-1,
        4.9468018558433052e-1,
        4.918898576307103e-1,
        4.4863432083118493e-1,
        4.5078099648970371e-1,
        4.1062102518688581e-1,
        3.602583146986339e-1,
        3.2757945699703028e-1,
        2.7928892704491243e-1,
        2.4985464964922635e-1,
        2.1080278995465099e-1,
        1.7799409692247298e-1,
        1.5411880069615053e-1,
        1.1957881962307169e-1,
        9.3993565751843353e-2,
        6.7190377322845676e-2,
        5.1366217946639786e-2,
        3.5716174780312381e-2,
        2.0672525560241309e-2,
        7.3004155738883077e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 24));
    }

    Ok(())
}

#[test]
fn dy_aa_ntuple_subgrid_dynamic() -> anyhow::Result<()> {
    // suppress LHAPDF banners
    lhapdf::set_verbosity(0);

    let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96);
    let mut grid = fill_drell_yan_lo_grid(&mut rng, 500_000, "NtupleSubgrid", true)?;

    grid.merge(fill_drell_yan_lo_grid(
        &mut rng,
        500_000,
        "NtupleSubgrid",
        true,
    )?)?;
    grid.scale(0.5);

    let pdf_set = "NNPDF31_nlo_as_0118_luxqed";

    assert!(lhapdf::available_pdf_sets().iter().any(|x| x == &pdf_set));

    let pdf = Pdf::with_setname_and_member(&pdf_set, 0);
    let xfx = |id, x, q2| pdf.xfx_q2(id, x, q2);
    let alphas = |_| 0.0;

    // check `read` and `write`
    let mut file = Cursor::new(Vec::new());
    grid.write(&mut file)?;
    file.set_position(0);
    mem::drop(grid);
    let mut grid = Grid::read(&mut file)?;

    // some useless scalings
    grid.scale_by_order(10.0, 0.5, 10.0, 10.0, 1.0);
    grid.scale_by_order(10.0, 1.0, 10.0, 10.0, 4.0);

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);

    let reference = vec![
        5.092821448721474e-1,
        5.191357484865172e-1,
        5.467704217708133e-1,
        4.8449484658306824e-1,
        4.728103049569273e-1,
        4.7483201886317034e-1,
        4.7061057035280723e-1,
        4.286036078931531e-1,
        4.2996760624782815e-1,
        3.9109949286253315e-1,
        3.430394818023909e-1,
        3.120393419396954e-1,
        2.6655870331253384e-1,
        2.3743619413745756e-1,
        2.005254408709319e-1,
        1.692014203393251e-1,
        1.463479865019976e-1,
        1.1348576426013411e-1,
        8.934108471882579e-2,
        6.385392511321722e-2,
        4.866767294756674e-2,
        3.379145451842768e-2,
        1.9493436557281443e-2,
        6.879891831200007e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    // check with `bin_indices` non-empty
    let other = grid.convolute(
        &xfx,
        &xfx,
        &alphas,
        &[],
        &(0..24).collect::<Vec<_>>(),
        &[],
        &[(1.0, 1.0)],
    );
    assert_eq!(bins, other);

    Ok(())
}

#[test]
fn dy_aa_lagrange_subgrid_dynamic() -> anyhow::Result<()> {
    // suppress LHAPDF banners
    lhapdf::set_verbosity(0);

    let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96);
    let mut grid = fill_drell_yan_lo_grid(&mut rng, 500_000, "LagrangeSubgrid", true)?;

    grid.merge(fill_drell_yan_lo_grid(
        &mut rng,
        500_000,
        "LagrangeSubgrid",
        true,
    )?)?;
    grid.scale(0.5);

    let pdf_set = "NNPDF31_nlo_as_0118_luxqed";

    assert!(lhapdf::available_pdf_sets().iter().any(|x| x == &pdf_set));

    let pdf = Pdf::with_setname_and_member(&pdf_set, 0);
    let xfx = |id, x, q2| pdf.xfx_q2(id, x, q2);
    let alphas = |_| 0.0;

    // check `read` and `write`
    let mut file = Cursor::new(Vec::new());
    grid.write(&mut file)?;
    file.set_position(0);
    mem::drop(grid);
    let mut grid = Grid::read(&mut file)?;

    // some useless scalings
    grid.scale_by_order(10.0, 0.5, 10.0, 10.0, 1.0);
    grid.scale_by_order(10.0, 1.0, 10.0, 10.0, 4.0);

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);
    let reference = vec![
        5.093090431949207e-1,
        5.191668797562395e-1,
        5.467930909144878e-1,
        4.845000146366871e-1,
        4.7279670245192884e-1,
        4.7481525429115445e-1,
        4.7060007393610065e-1,
        4.286009571276975e-1,
        4.2997427448811987e-1,
        3.911121604214181e-1,
        3.430494924416351e-1,
        3.1204501485640446e-1,
        2.6656633773109056e-1,
        2.3745772514449243e-1,
        2.0055415662593068e-1,
        1.692229208614707e-1,
        1.4635749293124972e-1,
        1.1348804559115269e-1,
        8.934261002200886e-2,
        6.385631160399488e-2,
        4.866968827833745e-2,
        3.379282482821324e-2,
        1.9494720220040448e-2,
        6.880478270711603e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    // optimize the grid
    grid.optimize();

    // check that the results are still the same
    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    Ok(())
}

#[test]
fn dy_aa_lagrange_subgrid_v2_dynamic() -> anyhow::Result<()> {
    // suppress LHAPDF banners
    lhapdf::set_verbosity(0);

    let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96);
    let mut grid = fill_drell_yan_lo_grid(&mut rng, 500_000, "LagrangeSubgridV2", true)?;

    grid.merge(fill_drell_yan_lo_grid(
        &mut rng,
        500_000,
        "LagrangeSubgridV2",
        true,
    )?)?;
    grid.scale(0.5);

    let pdf_set = "NNPDF31_nlo_as_0118_luxqed";

    assert!(lhapdf::available_pdf_sets().iter().any(|x| x == &pdf_set));

    let pdf = Pdf::with_setname_and_member(&pdf_set, 0);
    let xfx = |id, x, q2| pdf.xfx_q2(id, x, q2);
    let alphas = |_| 0.0;

    // check `read` and `write`
    let mut file = Cursor::new(Vec::new());
    grid.write(&mut file)?;
    file.set_position(0);
    mem::drop(grid);
    let mut grid = Grid::read(&mut file)?;

    // some useless scalings
    grid.scale_by_order(10.0, 0.5, 10.0, 10.0, 1.0);
    grid.scale_by_order(10.0, 1.0, 10.0, 10.0, 4.0);

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);
    let reference = vec![
        5.093090431949207e-1,
        5.191668797562395e-1,
        5.467930909144878e-1,
        4.845000146366871e-1,
        4.7279670245192884e-1,
        4.7481525429115445e-1,
        4.7060007393610065e-1,
        4.286009571276975e-1,
        4.2997427448811987e-1,
        3.911121604214181e-1,
        3.430494924416351e-1,
        3.1204501485640446e-1,
        2.6656633773109056e-1,
        2.3745772514449243e-1,
        2.0055415662593068e-1,
        1.692229208614707e-1,
        1.4635749293124972e-1,
        1.1348804559115269e-1,
        8.934261002200886e-2,
        6.385631160399488e-2,
        4.866968827833745e-2,
        3.379282482821324e-2,
        1.9494720220040448e-2,
        6.880478270711603e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    // optimize the grid
    //grid.optimize();

    // check that the results are still the same
    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    Ok(())
}

#[test]
fn dy_aa_lagrange_sparse_subgrid_dynamic() -> anyhow::Result<()> {
    // suppress LHAPDF banners
    lhapdf::set_verbosity(0);

    let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96);
    let mut grid = fill_drell_yan_lo_grid(&mut rng, 500_000, "LagrangeSparseSubgrid", true)?;

    grid.merge(fill_drell_yan_lo_grid(
        &mut rng,
        500_000,
        "LagrangeSparseSubgrid",
        true,
    )?)?;
    grid.scale(0.5);

    let pdf_set = "NNPDF31_nlo_as_0118_luxqed";

    assert!(lhapdf::available_pdf_sets().iter().any(|x| x == &pdf_set));

    let pdf = Pdf::with_setname_and_member(&pdf_set, 0);
    let xfx = |id, x, q2| pdf.xfx_q2(id, x, q2);
    let alphas = |_| 0.0;

    // check `read` and `write`
    let mut file = Cursor::new(Vec::new());
    grid.write(&mut file)?;
    file.set_position(0);
    mem::drop(grid);
    let mut grid = Grid::read(&mut file)?;

    // some useless scalings
    grid.scale_by_order(10.0, 0.5, 10.0, 10.0, 1.0);
    grid.scale_by_order(10.0, 1.0, 10.0, 10.0, 4.0);

    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);
    let reference = vec![
        5.093090431949207e-1,
        5.191668797562395e-1,
        5.467930909144878e-1,
        4.845000146366871e-1,
        4.7279670245192884e-1,
        4.7481525429115445e-1,
        4.7060007393610065e-1,
        4.286009571276975e-1,
        4.2997427448811987e-1,
        3.911121604214181e-1,
        3.430494924416351e-1,
        3.1204501485640446e-1,
        2.6656633773109056e-1,
        2.3745772514449243e-1,
        2.0055415662593068e-1,
        1.692229208614707e-1,
        1.4635749293124972e-1,
        1.1348804559115269e-1,
        8.934261002200886e-2,
        6.385631160399488e-2,
        4.866968827833745e-2,
        3.379282482821324e-2,
        1.9494720220040448e-2,
        6.880478270711603e-3,
    ];

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    // optimize the grid
    grid.optimize();

    // check that the results are still the same
    let bins = grid.convolute(&xfx, &xfx, &alphas, &[], &[], &[], &[(1.0, 1.0)]);

    for (result, reference) in bins.iter().zip(reference.iter()) {
        assert!(approx_eq!(f64, *result, *reference, ulps = 16));
    }

    Ok(())
}
