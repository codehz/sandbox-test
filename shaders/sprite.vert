#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in float radius;
layout(location = 0) out vec3 gcolor;
layout(location = 1) out float gradius;

layout(location = 0) uniform mat4 view_model;
layout(location = 1) uniform mat4 perspective;

void main() {
  gcolor = color;
  gl_Position = perspective * view_model * vec4(position, 1.0);
  vec4 another = perspective * view_model * vec4(position + vec3(1, 0, 0), 1.0);
  gradius = radius * length(another - gl_Position);
}