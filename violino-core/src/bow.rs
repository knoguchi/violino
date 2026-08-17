//! Bow-string friction: the nonlinearity that turns steady bow motion into
//! stick-slip oscillation (Helmholtz motion).

/// Friction curve after McIntyre-Schumacher-Woodhouse, in the simplified
/// bow-table form used by STK's `Bowed`: near `dv = 0` the bow sticks
/// (reflection coefficient 1), outside it slips with a steep `|dv|^-4`
/// falloff.
///
/// `dv` is bow velocity minus string velocity at the bow point. `slope`
/// controls the width of the sticking region and maps from bow pressure:
/// higher pressure = wider sticking region = lower slope value.
pub fn bow_friction(dv: f32, slope: f32) -> f32
{
    let x = (dv * slope).abs() + 0.75;
    x.powi(-4).min(1.0)
}
