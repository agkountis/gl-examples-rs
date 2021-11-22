#ifndef PBS_COMMON_GLSL_
#define PBS_COMMON_GLSL_

#include "assets/shaders/library/sampling_utils.glsl"

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
    vec2 texcoord;
#ifdef FEATURE_PARALLAX_MAPPING
    vec3 tViewDirection;
#endif
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

void CalculateTextureCoordinates(inout ShadingProperties props)
{
    props.texcoord = fsIn.texcoord;

    #ifdef FEATURE_PARALLAX_MAPPING
        // Choose Parallax Mapping method.
        if (parallaxMappingMethod == 1)
        {
            props.texcoord = ParallaxMapping(props.texcoord, props.tViewDirection);
        }
        else if (parallaxMappingMethod == 2)
        {
            props.texcoord = ParallaxMappingOffsetLimiting(props.texcoord, props.tViewDirection);
        }
        else if (parallaxMappingMethod == 3)
        {
            props.texcoord = SteepParallaxMapping(props.texcoord, props.tViewDirection);
        }
        else if (parallaxMappingMethod == 4)
        {
            props.texcoord = ParallaxOcclusionMapping(props.texcoord, props.tViewDirection);
        }

        // Discard fragments sampled outside the [0, 1] uv range. May cause artifacts when texture adressing is
        // set to repeat.
        if (props.texcoord.x > 1.0 || props.texcoord.y > 1.0 || props.texcoord.x < 0.0 || props.texcoord.y < 0.0)
        {
            discard;
        }
    #endif
}

void PopulateVectorProducts(inout ShadingProperties props)
{
    props.t = normalize(fsIn.wTangent.xyz);
    mat3 tangentToWorldMat = CreateTangentToWorldMatrix(normalize(fsIn.wNormal), props.t, fsIn.wTangent.w);

    vec3 v = normalize(fsIn.wViewDirection);

#ifdef FEATURE_PARALLAX_MAPPING
    mat3 worldToTangentMat = transpose(tangentToWorldMat);

    props.tViewDirection = normalize(worldToTangentMat * v);
    CalculateTextureCoordinates(props);
#endif

    // TODO: SampleNormalMap is not defined in this file. Fix this
    props.n = normalize(tangentToWorldMat * SampleNormalMap(normalMap, props.texcoord, 1.0));

    vec3 l = normalize(wLightDirection).xyz;
    vec3 h = normalize(l + v);
    props.r = reflect(-v, props.n);

    props.NoH = clamp(dot(props.n, h), 0.0, 1.0);

    props.NoV = clamp(abs(dot(props.n, v)), 0.0, 1.0);

    props.NoL = clamp(dot(props.n, l), 0.0, 1.0);
    props.HoV = clamp(dot(h, v), 0.0, 1.0);
}

#endif //PBS_COMMON_GLSL_