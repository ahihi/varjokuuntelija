#version 400

uniform vec2 u_resolution;
uniform float u_time;
uniform float u_midi_red;
uniform float u_midi_green;
uniform float u_midi_blue;

out vec4 frag_color;

#define TAU 6.283185307179586

vec2 polar(vec2 p_rect) {
    return vec2(atan(p_rect.y, p_rect.x), length(p_rect));
}

vec2 rect(vec2 p_polar) {
    return vec2(p_polar.y * cos(p_polar.x), p_polar.y * sin(p_polar.x));
}

vec2 rotate(float angle, vec2 p) {
    return rect(polar(p) + vec2(angle, 0.0));
}

vec4 circle(vec2 position, float radius, vec4 color, float stroke, vec4 stroke_color, vec2 p) {
    float dist = distance(p, position);
    
    if(dist < radius) {
        return color;
    } else if(dist < radius + stroke) {
        return stroke_color;
    } else {
        return vec4(0.0);
    }
}

vec4 rgb_circles(vec2 position, float radius, vec3 rgb, float stroke, vec4 stroke_color, float center_dist, vec2 p) {
    vec2 move = vec2(center_dist, 0.0);
    float angle_inc = TAU/3.0;
    
    vec4 color = vec4(0.0, 0.0, 0.0, 0.0);
            
    color += circle(
        position + move,
        radius, vec4(rgb.x, 0.0, 0.0, 1.0),
        stroke, stroke_color, p
    );
    color += circle(
        position + rotate(angle_inc, move),
        radius, vec4(0.0, rgb.y, 0.0, 1.0),
        stroke, stroke_color, p
    );
    color += circle(
        position + rotate(2.0*angle_inc, move),
        radius, vec4(0.0, 0.0, rgb.z, 1.0),
        stroke, stroke_color, p
    );
    
    return color;
}

void main() {
    float midi_red = u_midi_red / 127.0;
    float midi_green = u_midi_green / 127.0;
    float midi_blue = u_midi_blue / 127.0;
    
	vec2 p = 2.0*(gl_FragCoord.xy - 0.5 * u_resolution.xy) / u_resolution.xx;

    vec4 circles = rgb_circles(
        vec2(0.0, 0.0), 0.4, vec3(midi_red, midi_green, midi_blue),
        0.02, vec4(1.0, 1.0, 1.0, 1.0),
        0.25, rotate(u_time, p)
    );
    
    frag_color = circles;
}
