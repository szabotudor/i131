use engine131::systems::{System, SystemId};

#[derive(Default)]
pub(crate) struct Editor {}

impl System for Editor {
    fn initialize(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
        Ok(())
    }

    fn begin_play(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
        Ok(())
    }

    fn update(
        &mut self,
        engine: &engine131::I131,
        delta: f32,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = (engine, delta);
        Ok(())
    }

    fn in_editor_update(
        &mut self,
        engine: &engine131::I131,
        delta: f32,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = (engine, delta);
        Ok(())
    }

    fn end_play(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
        Ok(())
    }

    fn destroy(&mut self, engine: &engine131::I131) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
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
        SystemId("Editor131")
    }
}
