#version 450 core
#extension GL_ARB_separate_shader_objects : enable

const float EPSILON = 0.001;
const float F0_DIELECTRIC = 0.04;
const float PI = 3.14159265359;
const float ONE_OVER_PI = 0.318309886;

in VsOut {
    vec3 mLightDirection;
    vec3 mViewDirection;
    vec2 texcoord;
} fsIn;

layout(location = 0, binding = 0) uniform sampler2D albedoMap;
layout(location = 1, binding = 1) uniform sampler2D metallicMap;
layout(location = 2, binding = 2) uniform sampler2D roughnessMap;
layout(location = 3, binding = 3) uniform sampler2D normalMap;
layout(location = 4, binding = 4) uniform sampler2D aoMap;
layout(location = 5, binding = 5) uniform sampler2D brdfLUT;

layout(location = 6, binding = 6) uniform samplerCube irradianceMap;
layout(location = 7, binding = 7) uniform samplerCube radianceMap;

layout(location = 0) out vec4 outColor;

// TONEMAPPING FUNCTIONS ------------------------------------------------
const mat3 ACESInputMat =
{
    {0.59719, 0.35458, 0.04823},
    {0.07600, 0.90834, 0.01566},
    {0.02840, 0.13383, 0.83777}
};

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const mat3 ACESOutputMat =
{
    { 1.60475, -0.53108, -0.07367},
    {-0.10208,  1.10813, -0.00605},
    {-0.00327, -0.07276,  1.07602}
};

vec3 RRTAndODTFit(vec3 v)
{
    vec3 a = v * (v + 0.0245786f) - 0.000090537f;
    vec3 b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

// Complex fit. Better for realistic rendering
vec3 ACESFitted(vec3 color)
{
    color = ACESInputMat * color;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = ACESOutputMat * color;

    // Clamp to [0, 1]
    color = clamp(color, 0.0, 1.0);

    return color;
}

//simple luminance fit. Oversaturates brights
vec3 ACESFilm(vec3 x)
{
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}
// END TONEMAPPING FUNCTIONS --------------------------------------

// PBS FUNCTIONS --------------------------------------------------

// Analytical Lights---
vec3 FresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float DistributionGGX(float NdotH, float roughness)
{
    float a      = roughness * roughness;
    float a2     = a * a;
    float NdotH2 = NdotH * NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = roughness + 1.0;
    float k = r * r * 0.125; // 1.0 / 8.0 = 0.125

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float GeometrySmith(float NdotV, float NdotL, float roughness)
{
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 BRDF(vec3 N, vec3 V, vec3 L, vec3 F0, vec3 albedo, float metallic, float roughness)
{
    vec3 lightColor = vec3(1.0); //TODO: Replace with proper light uniforms
    vec3 H = normalize(L + V);

    float NdotH = clamp(dot(N, H), 0.0, 1.0);
    float NdotV = clamp(dot(N, V), 0.0, 1.0);
    float NdotL = clamp(dot(N, L), 0.0, 1.0);

    vec3 F = FresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);
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
vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 IBL(vec3 albedo, float ao)
{
    //TODO: TEMPORARY
    return vec3(0.03) * albedo * ao;
}
// --------------------

// END PBS FUNCTIONS ----------------------------------------------

void main()
{
    vec3 albedo = texture(albedoMap, fsIn.texcoord).rgb;

    float roughness = texture(roughnessMap, fsIn.texcoord).r;
    float metallic = texture(metallicMap, fsIn.texcoord).r;
    float ao = texture(aoMap, fsIn.texcoord).r;

    vec3 n = normalize(texture(normalMap, fsIn.texcoord).rgb * 2.0 - 1.0);

    vec3 v = normalize(fsIn.mViewDirection);
    vec3 l = normalize(fsIn.mLightDirection);


    vec3 F0 = mix(vec3(F0_DIELECTRIC), albedo, metallic);

    vec3 finalColor = BRDF(n, v, l, albedo, F0, roughness) + IBL(albedo, ao);

    // Tone map with ACES filter.
    outColor = vec4(ACESFitted(finalColor), albedo.a);
}