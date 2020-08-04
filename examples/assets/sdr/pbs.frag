#version 450 core
#extension GL_ARB_separate_shader_objects : enable

const float EPSILON = 0.001;
const float F0_DIELECTRIC = 0.04;
const float PI = 3.14159265359;
const float ONE_OVER_PI = 0.318309886;
const float MAX_REFLECTION_LOD = 5.0;
const float MIN_ROUGHNESS = 0.023;

in VsOut {
    vec3 wLightDirection;
    vec3 wViewDirection;
    vec3 wNormal;
    vec3 wTangent;
    vec2 texcoord;
} fsIn;

layout(location = 0, binding = 0) uniform sampler2D albedoMap;
layout(location = 1, binding = 1) uniform sampler2D normalMap;
layout(location = 2, binding = 2) uniform sampler2D m_r_aoMap;
layout(location = 3, binding = 3) uniform sampler2D brdfLUT;

layout(location = 4, binding = 4) uniform samplerCube irradianceMap;
layout(location = 5, binding = 5) uniform samplerCube radianceMap;

layout(location = 6) uniform vec3 wLightDirection;
layout(location = 7) uniform vec3 lightColor;

layout(location = 8) uniform vec3 baseColor;
layout(location = 9) uniform vec3 m_r_aoScale;

layout(location = 10) uniform int fresnelMode;
layout(location = 11) uniform vec3 m_r_ao_bias;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outBloomBrightColor;


mat3 CreateTBN(in vec3 n, in vec3 t)
{
    t = normalize(t - dot(t, n) * n);

    //Calculate the binormal
    vec3 b = normalize(cross(n, t));

    return mat3(t, b, n);
}

// PBS FUNCTIONS --------------------------------------------------

// Analytical Lights---
vec3 FresnelSchlick(in float cosTheta, in vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 FresnelSchlickRoughness(in float NdotV, in vec3 F0, in float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - NdotV, 5.0);
}

float DistributionGGX(in float NdotH, in float roughness)
{
    float a      = roughness * roughness;
    float a2     = a * a;
    float NdotH2 = NdotH * NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(in float NdotV, in float roughness)
{
    float r = roughness + 1.0;
    float k = (r * r) * 0.125; // 1.0 / 8.0 = 0.125

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float GeometrySmith(in float NdotV, in float NdotL, in float roughness)
{
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 BRDF(in float NdotH, in float NdotV, in float NdotL, in float HdotV, in vec3 lightColor, in vec3 F0, in vec3 albedo, in float metallic, in float roughness)
{
    vec3 F = FresnelSchlick(HdotV, F0);
    float NDF = DistributionGGX(NdotH, roughness);
    float G = GeometrySmith(NdotV, NdotL, roughness);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * NdotV * NdotL;

    vec3 specular = numerator / max(denominator, EPSILON);

    //Energy conservation
    vec3 kS = F;
    vec3 kD = (vec3(1.0) - kS) * (1.0 - metallic);

    return (kD * albedo * ONE_OVER_PI + specular) * lightColor * NdotL;
}

// --------------------

// IBL-----------------
vec3 EnvironmentBRDFApprox( vec3 F0, float roughness, float NoV )
{
    const vec4 c0 = vec4( -1, -0.0275, -0.572, 0.022 );
    const vec4 c1 = vec4( 1, 0.0425, 1.04, -0.04 );
    vec4 r = roughness * c0 + c1;
    float a004 = min( r.x * r.x, exp2( -9.28 * NoV ) ) * r.x + r.y;
    vec2 AB = vec2( -1.04, 1.04 ) * a004 + r.zw;
    return F0 * AB.x + AB.y;
}

vec3 IBL(in float NdotV, in vec3 F0, in vec3 albedo, in float metallic, in float roughness, in float ao, in vec2 brdfLUT, in vec3 irradiance, in vec3 radiance)
{
    if (fresnelMode == 0) {
        vec3 F = FresnelSchlickRoughness(NdotV, F0, roughness);

        vec3 kD = 1.0 - F;
        kD *= 1.0 - metallic;

        vec3 diffuse = irradiance * albedo;
        vec3 specular = radiance * (F * brdfLUT.x + brdfLUT.y);

        return (kD * diffuse + specular) * ao;
    }
    else if (fresnelMode == 1) {
        vec3 F = FresnelSchlickRoughness(NdotV, F0 * brdfLUT.x + brdfLUT.y, roughness);

        vec3 kD = 1.0 - F;
        kD *= 1.0 - metallic;

        vec3 diffuse = irradiance * albedo;
        vec3 specular = radiance * F;

        return (kD * diffuse + specular) * ao;
    }
    else if (fresnelMode == 2) {
        vec3 F = FresnelSchlickRoughness(NdotV, F0, roughness);

//        vec3 F = EnvironmentBRDFApprox(F0, roughness, NdotV);

//        vec3 F = F0 * brdfLUT.x + brdfLUT.y;

        vec3 kD = 1.0 - F;
        kD *= 1.0 - metallic;

        vec3 diffuse = irradiance * albedo;
        vec3 specular = radiance * F * EnvironmentBRDFApprox(F0, roughness, NdotV);

        return (kD * diffuse + specular) * ao;
    }

    return vec3(1.0);
}

// reference: https://github.com/google/filament/blob/main/shaders/src/light_indirect.fs
vec3 GetSpecularDominantDirection(const vec3 n, const vec3 r, in float roughness)
{
    return mix(r, n, roughness * roughness);
}

// reference: https://github.com/google/filament/blob/main/shaders/src/light_indirect.fs
float PerceptualRoughnessToLod(in float perceptualRoughness)
{
    return MAX_REFLECTION_LOD * perceptualRoughness * (2.0 - perceptualRoughness);
}
// --------------------

// END PBS FUNCTIONS ----------------------------------------------

float ConvertToGrayscale(in vec3 color)
{
    return dot(color, vec3(0.2125, 0.7154, 0.0721));
}

vec3 SampleNormalMap(in sampler2D normalMap, in vec2 texcoords, in float strength)
{
    vec3 norm = texture(normalMap, texcoords).rgb * 2.0 - 1.0;
    norm.xy *= strength;
    return norm;
}

void main()
{
    mat3 TBN = CreateTBN(normalize(fsIn.wNormal), normalize(fsIn.wTangent));

    vec3 n = normalize(TBN * SampleNormalMap(normalMap, fsIn.texcoord, 1.0));

    vec3 v = normalize(fsIn.wViewDirection);
    vec3 l = normalize(wLightDirection);
    vec3 h = normalize(l + v);
    vec3 r = reflect(-v, n);

    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float NdotV = clamp(dot(n, v), 0.0, 1.0);
    float NdotL = clamp(dot(n, l), 0.0, 1.0);
    float HdotV = clamp(dot(h, v), 0.0, 1.0);

    vec4 albedo = texture(albedoMap, fsIn.texcoord) * vec4(baseColor, 1.0);

    vec3 m_r_ao = texture(m_r_aoMap, fsIn.texcoord).rgb;
    float metallic = clamp(0.0, 1.0, m_r_ao.r + m_r_ao_bias.r) * m_r_aoScale.x;
    float roughness = clamp(MIN_ROUGHNESS, 1.0, m_r_ao.g + m_r_ao_bias.g) * m_r_aoScale.y;
    float ao = clamp(0.0, 1.0, m_r_ao.b + m_r_ao_bias.b) * m_r_aoScale.z;

    vec3 irradiance = texture(irradianceMap, n).rgb;

    float lod = PerceptualRoughnessToLod(roughness);
    vec3 specular_direction = GetSpecularDominantDirection(n, r, roughness);
    vec3 radiance = textureLod(radianceMap, specular_direction, lod).rgb;

    vec2 lutSample = texture(brdfLUT, vec2(NdotV, roughness)).rg;

    vec3 F0 = mix(vec3(F0_DIELECTRIC), albedo.rgb, metallic);

    vec3 finalColor = BRDF(NdotH, NdotV, NdotL, HdotV, lightColor, F0, albedo.rgb, metallic, roughness)
    + IBL(NdotV, F0, albedo.rgb, metallic, roughness, ao, lutSample, irradiance, radiance);

    outColor = vec4(finalColor, 1.0);

    float brightness = ConvertToGrayscale(finalColor);
    outBloomBrightColor = vec4(finalColor * step(1.0, brightness), 1.0) * 0.08;
}