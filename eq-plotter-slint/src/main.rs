use eq_plotter_slint::eq_plotter::EqPlotter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eq_plotter = EqPlotter::new()?;
    eq_plotter.run()?;
    Ok(())
}
