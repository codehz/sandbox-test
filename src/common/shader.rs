#[macro_export]
macro_rules! shader_program {
    ($display:expr, $shader:literal) => {
        glium::Program::from_source(
            $display,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".vert")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".frag")),
            None,
        )
    };
    ($display:expr, $shader:literal with geometry) => {
        glium::Program::from_source(
            $display,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".vert")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".frag")),
            Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".geom"))),
        )
    };
}

#[macro_export]
macro_rules! postprocess_shader_program {
    ($display:expr, $shader:literal) => {
        glium::Program::from_source(
            $display,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/postprocess.vert")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader, ".frag")),
            None,
        )
    };
}
