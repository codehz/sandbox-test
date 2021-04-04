#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in uint face;
layout(location = 0) out vec3 gcolor;
layout(location = 1) out uint gface;
layout(location = 2) out uint gpicked;
layout(binding = 0, std140) uniform picked { vec3 picked_position; };

void main() {
  gpicked = uint(position == picked_position);
  gcolor = color;
  gface = face;
  gl_Position = vec4(position, 0.0);
}