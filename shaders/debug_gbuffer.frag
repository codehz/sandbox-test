#version 450

layout(location = 0) uniform sampler2D color_sample;
layout(location = 1) uniform sampler2D normal_sample;
layout(location = 2) uniform sampler2D position_sample;

layout(location = 0) out vec4 color;

vec3 fetchPosition(ivec2 off) {
  vec2 resolution = textureSize(position_sample, 0);
  return texture(position_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}
vec3 fetchNormal(ivec2 off) {
  vec2 resolution = textureSize(position_sample, 0);
  return texture(normal_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}
vec3 fetchColor(ivec2 off) {
  vec2 resolution = textureSize(color_sample, 0);
  return texture(color_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}

vec3 get_score(vec3 curpos, vec3 curnorm, vec3 curcolor, ivec2 pos) {
  return vec3(length(curcolor - fetchColor(pos)),
              length(curnorm - fetchNormal(pos)),
              dot(curpos - fetchPosition(pos), curnorm));
}

void main() {
  vec3 normal = fetchPosition(ivec2(0, 0)) / 8.0 + 0.5;
  // float len = sqrt(pow(length(pos), 2) - pow(length(pos.xy), 2));
  color = vec4(normal, 1.0);
  // vec3 cur_position = fetchPosition(ivec2(0, 0));
  // vec3 cur_normal = fetchNormal(ivec2(0, 0));
  // vec3 cur_color = fetchColor(ivec2(0, 0));

  // vec3 score;

  // score += get_score(cur_position, cur_normal, cur_color, ivec2(-1, -1));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(-1, 0));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(-1, 1));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(0, -1));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(0, 1));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(1, -1));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(1, 0));
  // score += get_score(cur_position, cur_normal, cur_color, ivec2(1, 1));

  // color = vec4(mix(score, cur_color, 0.02), 1.0);
}