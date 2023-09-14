use smithay::{
    backend::renderer::gles::{
        element::PixelShaderElement, GlesPixelProgram, GlesRenderer, Uniform, UniformName,
        UniformType,
    },
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
        border_color: u32,
    ) -> PixelShaderElement {
        // Retrieve shader from EGL rendering context.
        let program = renderer
            .egl_context()
            .user_data()
            .get::<BorderShader>()
            .unwrap()
            .0
            .clone();

        let point = geo.size.to_point();

        // Colors are 24 bits with 8 bits for each red, green, blue value.
        // To get each color, shift the bits over by the offset and zero
        // out the other colors. The bitwise AND 255 does this because it will
        // zero out everything but the last 8 bits. This is where the color
        // has been shifted to.
        let red = border_color >> 16 & 255;
        let green = border_color >> 8 & 255;
        let blue = border_color & 255;

        let border_thickness = 2.0;

        PixelShaderElement::new(
            program,
            geo,
            None,
            alpha,
            vec![
                Uniform::new("u_resolution", (point.x as f32, point.y as f32)),
                Uniform::new("border_color", (red as f32, green as f32, blue as f32)),
                Uniform::new("border_thickness", border_thickness),
            ],
        )
    }
}

pub fn compile_shaders(renderer: &mut GlesRenderer) {
    // Compile GLSL file into pixel shader.
    let border_shader = renderer
        .compile_custom_pixel_shader(
            BORDER_SHADER,
            &[
                UniformName::new("u_resolution", UniformType::_2f),
                UniformName::new("border_color", UniformType::_3f),
                UniformName::new("border_thickness", UniformType::_1f),
            ],
        )
        .unwrap();

    // Save pixel shader in EGL rendering context.
    renderer
        .egl_context()
        .user_data()
        .insert_if_missing(|| BorderShader(border_shader));
}
