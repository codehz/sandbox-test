#version 450

layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

layout(location = 0) in vec3 gcolor[];
layout(location = 1) in uint gface[];
layout(location = 2) in uint gpicked[];
layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_position;
layout(location = 2) out uint v_picked;

layout(location = 0) uniform mat4 view_model;
layout(location = 1) uniform mat4 perspective;

// clang-format off
vec3 faces[24] = vec3[24](
  // North
  vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
  // South
  vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0),
  // East
  vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0),
  // West
  vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 1.0),
  // Up
  vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0),
  // Down
  vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0)
);
vec3 normals[6] = vec3[6](
  // North
  vec3(0.0, 0.0, -1.0),
  // South
  vec3(0.0, 0.0, 1.0),
  // East
  vec3(1.0, 0.0, 0.0),
  // West
  vec3(-1.0, 0.0, 0.0),
  // Up
  vec3(0.0, 1.0, 0.0),
  // Down
  vec3(0.0, -1.0, 0.0)
);
// clang-format on

void main() {
  v_picked = gpicked[0];
  v_color = gcolor[0];
  uint start = gface[0];
  vec4 source = gl_in[0].gl_Position;
  for (uint i = 0; i < 4; i++) {
    vec4 off = vec4(faces[i + start * 4], 1.0);
    vec4 real = source + off;
    vec4 sspos = view_model * real;
    gl_Position = perspective * sspos;
    v_position = sspos.xyz;
    EmitVertex();
  }
  EndPrimitive();
}