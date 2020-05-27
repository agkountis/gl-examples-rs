#version 450 core
#extension GL_ARB_separate_shader_objects : enable

const float EPSILON = 0.001;
const float F0_DIELECTRIC = 0.04;
const float PI = 3.14159265359;
const float ONE_OVER_PI = 0.318309886;
const float MAX_REFLECTION_LOD = 6.0;

in VsOut {
    vec3 wLightDirection;
    vec3 wViewDirection;
    vec3 tViewDirection;
    vec2 texcoord;
    mat3 TBN;
} fsIn;

layout(location = 0, binding = 0) uniform sampler2D albedoMap;
layout(location = 1, binding = 1) uniform sampler2D normalMap;
layout(location = 2, binding = 2) uniform sampler2D m_r_aoMap;
layout(location = 3, binding = 3) uniform sampler2D brdfLUT;
layout(location = 4, binding = 4) uniform sampler2D displacementMap;

layout(location = 5, binding = 5) uniform samplerCube irradianceMap;
layout(location = 6, binding = 6) uniform samplerCube radianceMap;

layout(location = 7) uniform vec3 wLightDirection;
layout(location = 8) uniform vec3 lightColor;
layout(location = 9) uniform int useParallax;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outBloomBrightColor;

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
    float k = (r * r) * 0.125; // 1.0 / 8.0 = 0.125

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

vec3 BRDF(float NdotH, float NdotV, float NdotL, float HdotV, vec3 lightColor, vec3 F0, vec3 albedo, float metallic, float roughness)
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
vec3 FresnelSchlickRoughness(float NdotV, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - NdotV, 5.0);
}

vec3 IBL(float NdotV, vec3 F0, vec3 albedo, float metallic, float roughness, float ao, vec2 brdfLUT, vec3 irradiance, vec3 radiance)
{
    vec3 F = FresnelSchlickRoughness(NdotV, F0, roughness);

    vec3 kS = F;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metallic;

    vec3 diffuse = irradiance * albedo;
    vec3 specular = radiance * (F * brdfLUT.x + brdfLUT.y);

    return (kD * diffuse + specular) * ao;
}
// --------------------

// END PBS FUNCTIONS ----------------------------------------------

float ConvertToGrayscale(vec3 color)
{
    return dot(color, vec3(0.2125, 0.7154, 0.0721));
}

vec3 SampleNormalMap(sampler2D normalMap, vec2 texcoords, float strength)
{
    vec3 norm = texture(normalMap, texcoords).rgb * 2.0 - 1.0;
    norm.xy *= strength;
    return norm;
}

// Reference: https://learnopengl.com/Advanced-Lighting/Parallax-Mapping
vec2 ParallaxOcclusionMapping(vec2 texcoord, vec3 viewDirection)
{
    // number of depth layers
    const float minLayers = 16;
    const float maxLayers = 64;
    const float heightScale = 0.012;

    // Calculate how many layers to use based on the angle of the Z axis in tangent space (points upwards)
    // and the view vector.
    float numLayers = mix(maxLayers, minLayers, abs(dot(vec3(0.0, 0.0, 1.0), viewDirection)));

    // calculate the size of each layer
    float layerDepth = 1.0 / numLayers;

    // depth of current layer
    float currentLayerDepth = 0.0;

    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDirection.xy / max(viewDirection.z, EPSILON) * heightScale;
    vec2 deltaTexCoords = P / numLayers;

    vec2  currentTexCoords     = texcoord;
    float currentDepthMapValue = texture(displacementMap, currentTexCoords).r;

    while(currentLayerDepth < currentDepthMapValue)
    {
        // shift texture coordinates along direction of P
        currentTexCoords -= deltaTexCoords;
        // get depthmap value at current texture coordinates
        currentDepthMapValue = texture(displacementMap, currentTexCoords).r;
        // get depth of next layer
        currentLayerDepth += layerDepth;
    }

    // get texture coordinates before collision (reverse operations)
    vec2 prevTexCoords = currentTexCoords + deltaTexCoords;

    // get depth after and before collision for linear interpolation
    float afterDepth  = currentDepthMapValue - currentLayerDepth;
    float beforeDepth = texture(displacementMap, prevTexCoords).r - currentLayerDepth + layerDepth;

    // interpolation of texture coordinates
    float weight = afterDepth / (afterDepth - beforeDepth);
    vec2 finalTexCoords = prevTexCoords * weight + currentTexCoords * (1.0 - weight);

    return finalTexCoords;
}

void main()
{
    vec2 texcoord = fsIn.texcoord;

    if (useParallax == 1) {
        texcoord = ParallaxOcclusionMapping(fsIn.texcoord, normalize(fsIn.tViewDirection));
    }

    if (texcoord.x > 1.0 || texcoord.y > 1.0 || texcoord.x < 0.0 || texcoord.y < 0.0)
    {
        discard;
    }

    vec3 n = normalize(fsIn.TBN * SampleNormalMap(normalMap, texcoord, 1.0));

    vec3 v = normalize(fsIn.wViewDirection);
    vec3 l = normalize(wLightDirection);
    vec3 h = normalize(l + v);
    vec3 r = reflect(-v, n);

    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float NdotV = clamp(dot(n, v), 0.0, 1.0);
    float NdotL = clamp(dot(n, l), 0.0, 1.0);
    float HdotV = clamp(dot(h, v), 0.0, 1.0);

    vec4 albedo = texture(albedoMap, texcoord);
    vec3 m_r_ao = texture(m_r_aoMap, texcoord).rgb;
    float metallic = m_r_ao.r;
    float roughness = m_r_ao.g;
    float ao = m_r_ao.b;

    vec3 irradiance = texture(irradianceMap, n).rgb;
    vec3 radiance = textureLod(radianceMap, r, roughness * MAX_REFLECTION_LOD).rgb;
    vec2 lutSample = texture(brdfLUT, vec2(NdotV, roughness)).rg;

    vec3 F0 = mix(vec3(F0_DIELECTRIC), albedo.rgb, metallic);

    vec3 finalColor = BRDF(NdotH, NdotV, NdotL, HdotV, lightColor, F0, albedo.rgb, metallic, roughness) +
                      IBL(NdotV, F0, albedo.rgb, metallic, roughness, ao, lutSample, irradiance, radiance);

    outColor = vec4(finalColor, 1.0);

    float brightness = ConvertToGrayscale(finalColor);

    outBloomBrightColor = vec4(finalColor * smoothstep(0.5, 1.0, brightness/brightness) * step(1.0, brightness), 1.0) * 0.08;
}