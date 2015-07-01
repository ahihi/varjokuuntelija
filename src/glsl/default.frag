#version 150
uniform vec2 u_resolution;
out vec4 out_color;

void main() {
    out_color = vec4(gl_FragCoord.x/u_resolution.x, 0.0, gl_FragCoord.y/u_resolution.y, 1.0);
}
