#version 450 core
#extension GL_ARB_separate_shader_objects : enable

const float EPSILON = 0.001;
const float F0_DIELECTRIC = 0.04;
const float PI = 3.14159265359;
const float ONE_OVER_PI = 0.318309886;
const float MAX_REFLECTION_LOD = 5.0;
const float MIN_ROUGHNESS = 0.023;

layout(location = 0) in VsOut {
    vec3 wViewDirection;
    vec3 wNormal;
    vec3 wTangent;
    vec2 texcoord;
} fsIn;

layout(std140, binding = 2) uniform PerFrameBlock
{
    vec4 wLightDirection;
    vec4 lightColor;
    vec2 ssVarianceAndThreshold;
    int specularAA;
    int disneyGgxHotness;
};

layout(std140, binding = 4) uniform MaterialBlock
{
    vec4 baseColor;
    float metallicScale;
    float metallicBias;
    float roughnessScale;
    float roughnessBias;
    float aoScale;
    float aoBias;
};

layout(binding = 0) uniform sampler2D albedoMap;
layout(binding = 1) uniform sampler2D normalMap;
layout(binding = 2) uniform sampler2D m_r_aoMap;
layout(binding = 3) uniform sampler2D brdfLUT;

layout(binding = 4) uniform samplerCube irradianceMap;
layout(binding = 5) uniform samplerCube radianceMap;


layout(location = 0) out vec4 outColor;


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

vec3 FresnelSchlickRoughness(in float NdotV, in vec3 F0, in float perceptualRoughness)
{
    return F0 + (max(vec3(1.0 - perceptualRoughness), F0) - F0) * pow(1.0 - NdotV, 5.0);
}

float DistributionGGX(in float NdotH, in float perceptualRoughness)
{
    float a      = perceptualRoughness * perceptualRoughness;
    float a2     = a * a;
    float NdotH2 = NdotH * NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

// Reference: http://www.jp.square-enix.com/tech/library/pdf/ImprovedGeometricSpecularAA(slides).pdf
// Reference: http://www.jp.square-enix.com/tech/library/pdf/ImprovedGeometricSpecularAA.pdf
float BiasedAxisAlignedGeometricSpecularAA(in vec3 tHalfVector, in float perceptualRoughness)
{
    float screenSpaceVariance = ssVarianceAndThreshold.x;
    float clampingThreshold = ssVarianceAndThreshold.y;

    float roughness = perceptualRoughness * perceptualRoughness;

    vec2 halfVector2D = tHalfVector.xy;
    vec2 deltaU = dFdx(halfVector2D);
    vec2 deltaV = dFdx(halfVector2D);

    vec2 boundingRectangle = abs(deltaU) + abs(deltaV);
    vec2 variance = screenSpaceVariance * (boundingRectangle * boundingRectangle);
    vec2 kernelRoughnessSquared = min(2.0 * variance, clampingThreshold);

    return clamp(roughness + kernelRoughnessSquared, 0.0, 1.0).x;
}

float DistributionGGXFiltered(in float NdotH, in float perceptualRoughness, in vec3 tHalfVector)
{
    float a      = BiasedAxisAlignedGeometricSpecularAA(tHalfVector, perceptualRoughness);
    float a2     = a * a;
    float NdotH2 = NdotH * NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(in float NdotV, in float roughness)
{
    float k = 0;

    // Disney's roughness remapping to reduce "hotness" for punctual lights.
    if (disneyGgxHotness == 1) {
        float r = roughness + 1.0;
        k = (r * r) * 0.125; // 1.0 / 8.0 = 0.125
    } else { // Default "k" value
        k = (roughness * roughness) * 0.5;
    }

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

vec3 BRDF(in float NdotH, in float NdotV, in float NdotL, in float HdotV, in vec3 lightColor,
in vec3 F0, in vec3 albedo, in float metallic, in float perceptualRoughness, in vec3 tHalfVector)
{
    vec3 F = FresnelSchlick(HdotV, F0);
    float NDF = 0.0;

    if (specularAA == 1) {
        NDF = DistributionGGXFiltered(NdotH, perceptualRoughness, tHalfVector);
    }
    else {
        NDF = DistributionGGX(NdotH, perceptualRoughness);
    }

    float G = GeometrySmith(NdotV, NdotL, perceptualRoughness);

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
    vec3 F = FresnelSchlickRoughness(NdotV, F0, roughness);

    vec3 kD = 1.0 - F;
    kD *= 1.0 - metallic;

    vec3 diffuse = irradiance * albedo;
    vec3 specular = radiance * (F0 * brdfLUT.x + brdfLUT.y);

    return (kD * diffuse + specular) * ao;
}

// reference: https://github.com/google/filament/blob/main/shaders/src/light_indirect.fs
vec3 GetSpecularDominantDirection(const vec3 n, const vec3 r, in float perceptualRoughness)
{
    return mix(r, n, perceptualRoughness * perceptualRoughness);
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
    vec3 l = normalize(wLightDirection).xyz;
    vec3 h = normalize(l + v);
    vec3 r = reflect(-v, n);

    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float NdotV = clamp(dot(n, v), 0.0, 1.0);
    float NdotL = clamp(dot(n, l), 0.0, 1.0);
    float HdotV = clamp(dot(h, v), 0.0, 1.0);

    vec4 albedo = texture(albedoMap, fsIn.texcoord) * vec4(baseColor.rgb, 1.0);

    vec3 m_r_ao = texture(m_r_aoMap, fsIn.texcoord).rgb;
    float metallic = clamp((m_r_ao.r + metallicBias) * metallicScale, 0.0, 1.0);
    float perceptualRoughness = clamp((m_r_ao.g + roughnessBias) * roughnessScale, MIN_ROUGHNESS, 1.0) ;
    float ao = clamp((m_r_ao.b + aoBias) * aoScale, 0.0, 1.0);

    vec3 irradiance = texture(irradianceMap, n).rgb;

    float lod = PerceptualRoughnessToLod(perceptualRoughness);
    vec3 specular_direction = GetSpecularDominantDirection(n, r, perceptualRoughness);
    vec3 radiance = textureLod(radianceMap, specular_direction, lod).rgb;

    vec2 lutSample = texture(brdfLUT, vec2(NdotV, perceptualRoughness)).rg;

    vec3 F0 = mix(vec3(F0_DIELECTRIC), albedo.rgb, metallic);

    vec3 finalColor = BRDF(NdotH, NdotV, NdotL, HdotV, lightColor.rgb, F0, albedo.rgb, metallic, perceptualRoughness, transpose(TBN) * h)
    + IBL(NdotV, F0, albedo.rgb, metallic, perceptualRoughness, ao, lutSample, irradiance, radiance);

    outColor = vec4(finalColor, 1.0);
}