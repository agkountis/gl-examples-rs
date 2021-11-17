#ifndef ENGINE_GLSL_
#define ENGINE_GLSL_

#define EPSILON                             1e-5
#define FP16_MAX                            65536.0
#define TRUE                                1
#define PI                                  3.14159265359
#define ONE_OVER_PI                         0.318309886

#define SAMPLER_2D(bind_slot, name) layout(binding = bind_slot) uniform sampler2D name
#define SAMPLER_CUBE(bind_slot, name) layout(binding = bind_slot) uniform samplerCube name

#define UNIFORM_BLOCK_BEGIN(bind_slot, name) layout(std140, binding = bind_slot) uniform name {
#define UNIFORM_BLOCK_END };
#define UNIFORM_BLOCK_END_NAMED(name) } name;

#define INPUT_BLOCK_BEGIN(loc, name) layout(location = loc) in VsOut {
#define INPUT_BLOCK_END };
#define INPUT_BLOCK_END_NAMED(name) } name;

#define OUTPUT(loc, name) layout(location = loc) out vec4 name

#define ERROR_COLOR vec4(1.0, 0.0, 1.0, 1.0)


#endif // ENGINE_GLSL_