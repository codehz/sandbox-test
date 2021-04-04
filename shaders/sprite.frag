#version 450

layout(location = 0) in vec3 v_color;
layout(location = 0) out vec4 sprite;

void main() {
  sprite = vec4(v_color, gl_FragCoord.w);
}