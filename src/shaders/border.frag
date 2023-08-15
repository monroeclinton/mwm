precision mediump float;
varying vec2 v_coords;

void main() {
    vec4 border = vec4(1.0, 0.0, 0.0, 1.0);
    vec4 transparent = vec4(0.0, 0.0, 0.0, 0.0);

    if (v_coords.x > 0.999 || v_coords.y > 0.999 || v_coords.x < 0.001 || v_coords.y < 0.001) {
        gl_FragColor = border;
    } else {
        gl_FragColor = transparent;
    }
}
