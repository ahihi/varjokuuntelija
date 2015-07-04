#version 150
uniform vec2 u_resolution;
uniform float u_time;
out vec4 out_color;

void main() {
    out_color = vec4(gl_FragCoord.x/u_resolution.x, 0.5+0.5*sin(u_time), gl_FragCoord.y/u_resolution.y, 1.0);
}
