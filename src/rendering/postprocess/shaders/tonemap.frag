#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "src/rendering/postprocess/shaders/include/tonemapping.glsl"

layout(binding = 0) uniform sampler2D image;

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

void main()
{
    vec3 color = texture(image, fsIn.texcoord).rgb * exposure;

    outColor = vec4(TONE_MAP(color), 1.0);
//    if (tonemappingOperator == 0) {
//        outColor = vec4(ACESFitted(color), 1.0);
//    } else if (tonemappingOperator == 1) {
//        outColor = vec4(ACESFilm(color), 1.0);
//    } else if (tonemappingOperator == 2) {
//        outColor = vec4(Reinhard(color), 1.0);
//    } else if (tonemappingOperator == 3) {
//        outColor = vec4(LumaBasedReinhard(color), 1.0);
//    } else if (tonemappingOperator == 4) {
//        outColor = vec4(WhitePreservingLumaBasedReinhard(color), 1.0);
//    } else if (tonemappingOperator == 5) {
//        outColor = vec4(Uncharted2(color), 1.0);
//    } else if (tonemappingOperator == 6) {
//        outColor = vec4(RomBinDaHouse(color), 1.0);
//    } else {
////        outColor = vec4(ACESFitted(color), 1.0);
//        outColor = vec4(1.0, 0.0, 0.0, 1.0);
//    }
//    outColor = vec4(color, 1.0);
}
