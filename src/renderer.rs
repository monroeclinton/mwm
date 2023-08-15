use smithay::{
    backend::renderer::gles::{element::PixelShaderElement, GlesPixelProgram, GlesRenderer},
    utils::{Logical, Rectangle},
};

const BORDER_SHADER: &str = include_str!("shaders/border.frag");

// Define a struct that holds a pixel shader. This struct will be stored in the data of the
// EGL rendering context.
pub struct BorderShader(pub GlesPixelProgram);

impl BorderShader {
    pub fn element(
        renderer: &GlesRenderer,
        geo: Rectangle<i32, Logical>,
        alpha: f32,
    ) -> PixelShaderElement {
        // Retrieve shader from EGL rendering context.
        let program = renderer
            .egl_context()
            .user_data()
            .get::<BorderShader>()
            .unwrap()
            .0
            .clone();

        PixelShaderElement::new(program, geo, None, alpha, vec![])
    }
}

pub fn compile_shaders(renderer: &mut GlesRenderer) {
    // Compile GLSL file into pixel shader.
    let border_shader = renderer
        .compile_custom_pixel_shader(BORDER_SHADER, &[])
        .unwrap();

    // Save pixel shader in EGL rendering context.
    renderer
        .egl_context()
        .user_data()
        .insert_if_missing(|| BorderShader(border_shader));
}
