use engine131::{I131, schedulers::DAGScheduler};

trait ResultPrint<T> {
    fn print(self) -> Result<T, i32>;
}
impl<T, E: std::fmt::Display> ResultPrint<T> for Result<T, E> {
    fn print(self) -> Result<T, i32> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                eprintln!("{err}");
                Err(-1)
            }
        }
    }
}

fn main() -> Result<(), i32> {
    let engine = I131::new(3, Box::new(DAGScheduler::new())).print()?;
    println!("Created engine");

    engine.initialize().print()?;
    println!("Initialized engine");

    println!("Created test system");

    engine.main_loop().print()?;

    Ok(())
}
