#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/engine.glsl"

#define MIN_ROUGHNESS                       0.045

#define RENDER_MODE_ALBEDO                  1
#define RENDER_MODE_METALLIC                2
#define RENDER_MODE_ROUGHNESS               3
#define RENDER_MODE_NORMALS                 4
#define RENDER_MODE_TANGENTS                5
#define RENDER_MODE_UV                      6
#define RENDER_MODE_NDOTV                   7
#define RENDER_MODE_AO                      8
#define RENDER_MODE_SPECULAR_AO             9
#define RENDER_MODE_HORIZON_SPECULAR_AO     10
#define RENDER_MODE_DIFFUSE_AMBIENT         11
#define RENDER_MODE_SPECULAR_AMBIENT        12
#define RENDER_MODE_FRESNEL                 13
#define RENDER_MODE_FRESNEL_RADIANCE        14
#define RENDER_MODE_ANALYTICAL_LIGHTS_ONLY  15
#define RENDER_MODE_IBL_ONLY                16

INPUT_BLOCK_BEGIN(0, VsOut)
    vec3 wViewDirection;
    vec3 wNormal;
    vec4 wTangent;
    vec2 texcoord;
INPUT_BLOCK_END_NAMED(fsIn)

UNIFORM_BLOCK_BEGIN(2, PerFrameBlock)
    vec4 wLightDirection;
    vec4 lightColor;
    vec2 ssVarianceAndThreshold;
    int specularAA;
    int specularAO;
    int renderMode;
    int mulriScattering;
    float maxReflectionLod;
UNIFORM_BLOCK_END

UNIFORM_BLOCK_BEGIN(4, MaterialBlock)
    vec4 baseColor;
    float metallicScale;
    float metallicBias;
    float roughnessScale;
    float roughnessBias;
    float aoScale;
    float aoBias;
    float reflectance;
#ifdef FEATURE_PARALLAX_MAPPING
    float pomMinLayers;
    float pomMaxLayers;
    float pomDisplacementScale;
    int parallaxMappingMethod;
#endif // FEATURE_PARALLAX_MAPPING
UNIFORM_BLOCK_END

SAMPLER_2D(0, albedoMap);
SAMPLER_2D(1, normalMap);
SAMPLER_2D(2, m_r_aoMap);
SAMPLER_2D(3, brdfLUT);

SAMPLER_CUBE(4, irradianceMap);
SAMPLER_CUBE(5, radianceMap);

#ifdef FEATURE_PARALLAX_MAPPING
    SAMPLER_2D(6, displacementMap);
#endif

OUTPUT(0, outColor);

#ifdef FEATURE_PARALLAX_MAPPING
    #include "assets/shaders/library/parallax_mapping.glsl"
#endif // FEATURE_PARALLAX_MAPPING

#include "assets/shaders/library/pbs_common.glsl"

#include "assets/shaders/library/brdf.glsl"
#include "assets/shaders/library/ibl.glsl"

float ConvertToGrayscale(in vec3 color)
{
    return dot(color, vec3(0.2125, 0.7154, 0.0721));
}

void CalculateF0(inout ShadingProperties props)
{
    props.F0 = 0.16 * reflectance * reflectance * (1.0 - props.metallic) + props.albedo.rgb * props.metallic;
}

void PopulateIBLProperties(inout ShadingProperties props)
{
    props.irradiance = texture(irradianceMap, props.n).rgb;

    float lod = PerceptualRoughnessToLod(props.perceptualRoughness);
    vec3 specular_direction = GetSpecularDominantDirection(props.n, props.r, props.roughness);
    props.radiance = textureLod(radianceMap, specular_direction, lod).rgb;

    props.brdfLUT = texture(brdfLUT, vec2(props.NoV, props.perceptualRoughness)).rg;
}

void PopulateMaterialProperties(inout ShadingProperties props)
{
    props.albedo = texture(albedoMap, props.texcoord) * vec4(baseColor.rgb, 1.0);

    vec3 m_r_ao = texture(m_r_aoMap, props.texcoord).rgb;
    props.metallic = clamp((m_r_ao.r + metallicBias) * metallicScale, 0.0, 1.0);
    props.perceptualRoughness = clamp((m_r_ao.g + roughnessBias) * roughnessScale, MIN_ROUGHNESS, 1.0) ;

    if (specularAA == 1) {
        props.perceptualRoughness = BiasedAxisAlignedGeometricSpecularAA(props.n, props.perceptualRoughness);
    }

    props.roughness = PerceptualRoughnessToRoughness(props.perceptualRoughness);
    props.ao = clamp((m_r_ao.b + aoBias) * aoScale, 0.0, 1.0);

    if (specularAO == 1) {
        props.so = ComputeSpecularAO(props.NoV, props.ao, props.roughness);
        props.horizonSo = ComputeHorizonSpecularAO(props.r, props.n);
    }
}

void PopulateShadingProperties(out ShadingProperties props)
{
    PopulateVectorProducts(props);
    PopulateMaterialProperties(props);
    PopulateIBLProperties(props);
    CalculateF0(props);
}

vec4 ComputeOutputColor(in ShadingProperties props)
{
    vec3 analyticalLight = BRDF(props);

    vec3 imageBasedLight = IBL(props);

    switch (renderMode) {
        case RENDER_MODE_ALBEDO:
            return vec4(props.albedo.rgb, 1.0);
        case RENDER_MODE_METALLIC:
            return vec4(props.metallic.rrr, 1.0);
        case RENDER_MODE_ROUGHNESS:
            return vec4(props.perceptualRoughness.rrr, 1.0);
        case RENDER_MODE_NORMALS:
            return vec4(props.n * 0.5 + 0.5, 1.0);
        case RENDER_MODE_TANGENTS:
            return vec4(props.t * 0.5 + 0.5, 1.0);
        case RENDER_MODE_UV:
            return vec4(props.texcoord, 0.0, 1.0);
        case RENDER_MODE_NDOTV:
            return vec4(props.NoV.xxx, 1.0);
        case RENDER_MODE_AO:
            return vec4(props.ao.rrr, 1.0);
        case RENDER_MODE_SPECULAR_AO:
            return vec4(props.so.rrr, 1.0);
        case RENDER_MODE_HORIZON_SPECULAR_AO:
            return vec4(props.horizonSo.xxx, 1.0);
        case RENDER_MODE_DIFFUSE_AMBIENT:
            return vec4(props.irradiance.rgb, 1.0);
        case RENDER_MODE_SPECULAR_AMBIENT:
            return vec4(props.radiance.rgb, 1.0);
        case RENDER_MODE_FRESNEL:
            return vec4(FresnelSchlick(props.NoV, props.F0), 1.0);
        case RENDER_MODE_FRESNEL_RADIANCE:
            return vec4(props.radiance * (props.F0 * props.brdfLUT.x + props.brdfLUT.y), 1.0);
        case RENDER_MODE_ANALYTICAL_LIGHTS_ONLY:
            return vec4(analyticalLight, 1.0);
        case RENDER_MODE_IBL_ONLY:
            return vec4(imageBasedLight, 1.0);
        default:
            return vec4(analyticalLight + imageBasedLight, 1.0);
    }
}

void main()
{
    ShadingProperties shadingProps;
    PopulateShadingProperties(shadingProps);
    outColor = ComputeOutputColor(shadingProps);
}