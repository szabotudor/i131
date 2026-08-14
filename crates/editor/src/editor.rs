use std::{cell::RefCell, mem::offset_of, rc::Rc};

use engine131::{
    math131::{Vec2f32, Vec2u32, Vec3f32},
    renderer131::{
        BufferBinding, BufferCreateInfo, BufferFieldFormat, BufferHandle, BufferUsage,
        ComponentBitCount, DrawCall, ProgramHandle, Renderer, RendererError, ScalarKind,
        ShaderCreateInfo, ShaderHandle, ShaderStage,
    },
    systems::{System, SystemContext, SystemId},
    window131::{Window, WindowError, WindowMode, WindowSettings},
};
use thiserror::Error;
use vulkan_renderer::{ValidationLevel, VulkanRenderer};

use crate::shaders_vulkan;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Window error: {0}")]
    WindowError(#[from] WindowError),

    #[error("Window error: {0}")]
    RendererError(#[from] RendererError),
}

pub(crate) struct Editor {
    context: SystemContext,
    renderer: Box<dyn Renderer>,
    window: Rc<RefCell<Window>>,
    vert: ShaderHandle,
    frag: ShaderHandle,
    prog: ProgramHandle,
    vertices: BufferHandle,
}
unsafe impl Send for Editor {}
unsafe impl Sync for Editor {}

impl Editor {
    pub fn new() -> Result<Self, EditorError> {
        let window = Rc::new(RefCell::new(Window::new(
            WindowSettings::new()
                .with_title("I131".to_string())
                .with_size(Vec2u32::new(800, 600))
                .with_mode(WindowMode::Windowed),
        )?));
        // TODO: Implement system init arguments
        //
        // Needed to enable/disable validation here
        let renderer = VulkanRenderer::new_glfw(
            "I131_VulkanBackend",
            (1, 3, 0),
            window.clone(),
            ValidationLevel::Normal,
        )?;

        Ok(Self {
            context: SystemContext::empty(),
            window,
            renderer: Box::new(renderer),
            vert: ShaderHandle::null(),
            frag: ShaderHandle::null(),
            prog: ProgramHandle::null(),
            vertices: BufferHandle::null(),
        })
    }
}

#[repr(C)]
struct Vertex {
    pub pos: Vec2f32,
    pub col: Vec3f32,
}

impl System for Editor {
    const SYSTEM_ID: SystemId = SystemId("Editor");
    const DEPENDENCIES: &'static [SystemId] = &[];
    const BEFORE: &'static [SystemId] = &[];
    const AFTER: &'static [SystemId] = &[];

    fn initialize(
        &mut self,
        context: SystemContext,
    ) -> Result<(), engine131::systems::SystemError> {
        self.context = context;

        let default_vert = self.renderer.create_shader(ShaderCreateInfo {
            source: shaders_vulkan::DEFAULT_VERT,
            stage: ShaderStage::Vertex,
            name: "default.vert".to_string(),
        })?;
        let default_frag = self.renderer.create_shader(ShaderCreateInfo {
            source: shaders_vulkan::DEFAULT_FRAG,
            stage: ShaderStage::Pixel,
            name: "default.frag".to_string(),
        })?;

        self.vert = default_vert;
        self.frag = default_frag;

        let default_prog = self
            .renderer
            .create_program(&[default_vert, default_frag])?;

        self.prog = default_prog;

        let vertices = [
            ((0.0, -0.5), (1.0, 1.0, 1.0)),
            ((0.5, 0.5), (0.0, 1.0, 0.0)),
            ((-0.5, 0.5), (0.0, 0.0, 1.0)),
        ]
        .map(|(v, c)| Vertex {
            pos: Vec2f32::new(v.0, v.1),
            col: Vec3f32::new(c.0, c.1, c.2),
        });

        let handle = self.renderer.create_buffer(
            BufferCreateInfo::new(BufferUsage::Vertex, &vertices)
                .with_field::<Vec2f32>(
                    BufferBinding::Location(0),
                    offset_of!(Vertex, pos),
                    BufferFieldFormat {
                        kind: ScalarKind::Float,
                        normalized: false,
                        bits_per_component: ComponentBitCount::Two { a: 32, b: 32 },
                    },
                )
                .with_field::<Vec2f32>(
                    BufferBinding::Location(1),
                    offset_of!(Vertex, col),
                    BufferFieldFormat {
                        kind: ScalarKind::Float,
                        normalized: false,
                        bits_per_component: ComponentBitCount::Three {
                            r: 32,
                            g: 32,
                            b: 32,
                        },
                    },
                ),
        )?;

        self.vertices = handle;

        Ok(())
    }

    fn begin_play(&mut self) -> Result<(), engine131::systems::SystemError> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> Result<(), engine131::systems::SystemError> {
        let _ = delta;
        let engine = self.context.engine()?;
        let mut window = self.window.borrow_mut();

        window.update();

        if window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        drop(window);

        self.renderer.execute(DrawCall::Draw {
            program: self.prog,
            vertex_buffers: vec![self.vertices],
        })?;

        Ok(())
    }

    fn in_editor_update(&mut self, delta: f32) -> Result<(), engine131::systems::SystemError> {
        let _ = delta;
        let engine = self.context.engine()?;
        let mut window = self.window.borrow_mut();

        window.update();

        if window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        drop(window);

        Ok(())
    }

    fn end_play(&mut self) -> Result<(), engine131::systems::SystemError> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), engine131::systems::SystemError> {
        self.renderer.destroy()?;
        self.vert = ShaderHandle::null();
        self.frag = ShaderHandle::null();
        self.prog = ProgramHandle::null();

        Ok(())
    }
}
