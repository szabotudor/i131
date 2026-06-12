use engine131::{
    I131,
    schedulers::DAGScheduler,
    systems::{System, SystemId},
};

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

struct Test0 {
    count: f32,
}
impl System for Test0 {
    fn initialize(&mut self, _engine: &I131) -> Result<(), engine131::systems::SystemError> {
        println!("Initialized system");
        Ok(())
    }

    fn begin_play(&mut self, _engine: &I131) -> Result<(), engine131::systems::SystemError> {
        println!("Begin play");
        Ok(())
    }

    fn update(&mut self, engine: &I131, delta: f32) -> Result<(), engine131::systems::SystemError> {
        println!("Update {}", self.count);
        self.count += delta;
        if self.count > 5.0 {
            engine.destroy_system(Self::system_id())?;
        }
        Ok(())
    }

    fn in_editor_update(
        &mut self,
        _engine: &I131,
        delta: f32,
    ) -> Result<(), engine131::systems::SystemError> {
        println!("Update(editor) {}", self.count);
        self.count += delta;
        Ok(())
    }

    fn end_play(&mut self, _engine: &I131) -> Result<(), engine131::systems::SystemError> {
        println!("End play");
        Ok(())
    }

    fn destroy(&mut self, _engine: &I131) -> Result<(), engine131::systems::SystemError> {
        println!("Destroy system");
        Ok(())
    }

    fn dependencies() -> &'static [engine131::systems::SystemId]
    where
        Self: Sized,
    {
        &[]
    }

    fn system_id() -> engine131::systems::SystemId
    where
        Self: Sized,
    {
        SystemId("Test0")
    }
}

fn main() -> Result<(), i32> {
    let engine = I131::new(3, Box::new(DAGScheduler::new())).print()?;
    println!("Created engine");

    engine.initialize().print()?;
    println!("Initialized engine");

    engine.create_system(Test0 { count: 0.0 }).print()?;
    println!("Created test system");

    engine.main_loop().print()?;

    Ok(())
}
