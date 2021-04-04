#version 450

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec3 v_position;
layout(location = 2) flat in uint v_picked;
layout(location = 0) out vec4 color;
layout(location = 1) out vec3 normal;
layout(location = 2) out vec3 position;

void main() {
  color = vec4(v_color, float(v_picked));
  normal = normalize(cross(dFdx(v_position), dFdy(v_position)));
  position = v_position.xyz;
}
