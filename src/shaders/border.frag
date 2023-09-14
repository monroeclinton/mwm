precision mediump float;
// The size or dimensions.
uniform vec2 u_resolution;
// Color of border
uniform vec3 border_color;
// The ratio of the coordinate to the resolution.
varying vec2 v_coords;

void main() {
    float thickness = 2.0;

    // Get the pixel coordinates.
    vec2 coords = v_coords * u_resolution;

    // Step function is just (param1 < param2) return 1.0 for true and 0.0 for false.
    // On the left side, if the coordinate is less than the thickness, draw a border.
    float xl = step(coords.x, thickness);
    float yl = step(coords.y, thickness);
    // On the right side, if (coordinate - thickness) is less than the coordinate, draw a border.
    float xr = step(u_resolution.x - thickness, coords.x);
    float yr = step(u_resolution.y - thickness, coords.y);

    // The alpha will become 1.0 or greater if any of the above statements are true.
    float alpha = xl + yl + xr + yr;

    gl_FragColor = vec4(border_color * alpha, alpha);
}
