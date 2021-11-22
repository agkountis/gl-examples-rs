#ifndef SAMPLING_UTILS_GLSL_
#define SAMPLING_UTILS_GLSL_

vec3 SampleNormalMap(in sampler2D normalMap, in vec2 texcoords, in float strength)
{
    vec3 norm = texture(normalMap, texcoords).rgb * 2.0 - 1.0;
    norm.xy *= strength;
    return norm;
}

#endif // SAMPLING_UTILS_GLSL_