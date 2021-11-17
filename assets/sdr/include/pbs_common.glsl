#ifndef PBS_COMMON_GLSL_
#define PBS_COMMON_GLSL_

struct ShadingProperties {
    vec4 albedo;
    float perceptualRoughness;
    float roughness;
    float metallic;
    float ao;
    float so;
    float horizonSo;
    vec2 brdfLUT;
    vec3 F0;
    vec3 irradiance;
    vec3 radiance;
    float NoV;
    float NoL;
    float NoH;
    float HoV;
    vec3 n;
    vec3 r;
    vec3 t;
};

mat3 CreateTangentToWorldMatrix(in vec3 n, in vec3 t, in float tSign)
{
    t = normalize(t - dot(t, n) * n);

    //Calculate the binormal
    vec3 b = normalize(cross(n, t) * tSign);

    return mat3(t, b, n);
}

float PerceptualRoughnessToRoughness(float perceptualRoughness)
{
    return perceptualRoughness * perceptualRoughness;
}

float RoughnessToPerceptualRoughness(float roughness)
{
    return sqrt(roughness);
}

#endif //PBS_COMMON_GLSL_