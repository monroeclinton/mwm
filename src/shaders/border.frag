precision mediump float;
uniform vec2 u_resolution;
varying vec2 v_coords;

void main() {
    float thickness = 2.0;
    vec4 color = vec4(1.0, 0.0, 0.0, 1.0);
    vec4 transparent = vec4(0.0, 0.0, 0.0, 0.0);
    vec2 coords = v_coords * u_resolution;

    if (
        coords.x >= (u_resolution.x - thickness) ||
        coords.y >= (u_resolution.y - thickness) ||
        coords.x <= thickness || coords.y <= thickness
    ) {
        gl_FragColor = color;
    } else {
        gl_FragColor = transparent;
    }
}
