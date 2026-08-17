//! Python bindings: the research-iteration interface.
//!
//! Build with maturin: `maturin develop` inside `violino-py/`.

use pyo3::prelude::*;

/// A violin voice driven by per-sample control curves.
#[pyclass]
struct Violin
{
    inner: violino_core::Violin,
}

#[pymethods]
impl Violin
{
    #[new]
    fn new(sample_rate: f32) -> Self
    {
        Violin { inner: violino_core::Violin::new(sample_rate) }
    }

    /// Render audio from per-sample control curves (all the same length):
    /// fundamental in Hz, bow velocity (~0..0.4), bow pressure (0..1).
    fn render(&mut self, f0: Vec<f32>, bow_velocity: Vec<f32>, bow_pressure: Vec<f32>) -> PyResult<Vec<f32>>
    {
        let n = f0.len();
        if bow_velocity.len() != n || bow_pressure.len() != n
        {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "f0, bow_velocity and bow_pressure must have the same length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n
        {
            out.push(self.inner.tick(f0[i], bow_velocity[i], bow_pressure[i]));
        }
        Ok(out)
    }
}

#[pymodule]
fn violino(m: &Bound<'_, PyModule>) -> PyResult<()>
{
    m.add_class::<Violin>()?;
    Ok(())
}
