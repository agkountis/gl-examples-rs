#version 450 core
#extension GL_ARB_separate_shader_objects : enable

const float EPSILON = 1e-5;
const float F0_DIELECTRIC = 0.04;
const float PI = 3.14159265359;
const float ONE_OVER_PI = 0.318309886;
const float MIN_ROUGHNESS = 0.045;

const int RENDER_MODE_ALBEDO = 1;
const int RENDER_MODE_METALLIC = 2;
const int RENDER_MODE_ROUGHNESS = 3;
const int RENDER_MODE_NORMALS = 4;
const int RENDER_MODE_TANGENTS = 5;
const int RENDER_MODE_UV = 6;
const int RENDER_MODE_NDOTV = 7;
const int RENDER_MODE_AO = 8;
const int RENDER_MODE_SPECULAR_AO = 9;
const int RENDER_MODE_HORIZON_SPECULAR_AO = 10;
const int RENDER_MODE_DIFFUSE_AMBIENT = 11;
const int RENDER_MODE_SPECULAR_AMBIENT = 12;
const int RENDER_MODE_FRESNEL = 13;
const int RENDER_MODE_FRESNEL_RADIANCE = 14;
const int RENDER_MODE_ANALYTICAL_LIGHTS_ONLY = 15;
const int RENDER_MODE_IBL_ONLY = 16;

const int BRDF_FILLAMENT = 0;
const int BRDF_UE4 = 1;

layout(location = 0) in VsOut {
    vec3 wViewDirection;
    vec3 wNormal;
    vec4 wTangent;
    vec2 texcoord;
} fsIn;

layout(std140, binding = 2) uniform PerFrameBlock
{
    vec4 wLightDirection;
    vec4 lightColor;
    vec2 ssVarianceAndThreshold;
    int specularAA;
    int specularAO;
    int renderMode;
    int brdfType;
    int mulriScattering;
    float maxReflectionLod;
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
    float reflectance;
};

layout(binding = 0) uniform sampler2D albedoMap;
layout(binding = 1) uniform sampler2D normalMap;
layout(binding = 2) uniform sampler2D m_r_aoMap;
layout(binding = 3) uniform sampler2D brdfLUT;

layout(binding = 4) uniform samplerCube irradianceMap;
layout(binding = 5) uniform samplerCube radianceMap;

layout(location = 0) out vec4 outColor;

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

// PBS FUNCTIONS --------------------------------------------------

// Analytical Lights---
vec3 FresnelSchlick(in float NdotV, in vec3 F0)
{
    vec3 F90 = vec3(1.0);

    if (specularAO == 1) {
        F90 = vec3(clamp(dot(F0, vec3(50.0 * 0.33)), 0.0, 1.0));
    }

    return F0 + (F90 - F0) * pow(1.0 - NdotV, 5.0);
}

float DistributionGGX(in float NdotH, in float roughness)
{
    float a = roughness;
    float a2 = a * a;
    float NdotH2 = NdotH * NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

// Fillament
float D_GGX(float NoH, float a) {
    float a2 = a * a;
    float f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

// Reference: http://www.jp.square-enix.com/tech/library/pdf/ImprovedGeometricSpecularAA(slides).pdf
// Reference: http://www.jp.square-enix.com/tech/library/pdf/ImprovedGeometricSpecularAA.pdf
float BiasedAxisAlignedGeometricSpecularAA(in vec3 wNormal, in float perceptualRoughness)
{
    vec3 du = dFdx(wNormal);
    vec3 dv = dFdy(wNormal);

    float variance = ssVarianceAndThreshold.x * (dot(du, du) + dot(dv, dv));

    float roughness = PerceptualRoughnessToRoughness(perceptualRoughness);
    float kernelRoughness = min(2.0 * variance, ssVarianceAndThreshold.y);
    float squareRoughness = clamp(roughness * roughness + kernelRoughness, 0.0, 1.0);

    return RoughnessToPerceptualRoughness(sqrt(squareRoughness));
}

float ComputeSpecularAO(float NoV, float ao, float roughness)
{
    return clamp(pow(NoV + ao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ao, 0.0, 1.0);
}

float ComputeHorizonSpecularAO(vec3 r, vec3 n)
{
    float horizon = min(1.0 + dot(r, n), 1.0);
    return horizon * horizon;
}

float G_SchlickGGX(in float NdotV, in float perceptualRoughness)
{
    float r = perceptualRoughness + 1.0;
    float k = (r * r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

// UE4
float V_SmithSchlickGGX(in float NdotV, in float NdotL, in float roughness)
{
    float ggx2  = G_SchlickGGX(NdotV, roughness);
    float ggx1  = G_SchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

float V_SmithGGXCorrelated(float NoV, float NoL, float roughness)
{
    float a2 = roughness * roughness;
    float lambdaV = NoL * sqrt((NoV - a2 * NoV) * NoV + a2);
    float lambdaL = NoV * sqrt((NoL - a2 * NoL) * NoL + a2);
    return 0.5 / (lambdaV + lambdaL);
}

float V_SmithGGXCorrelatedFast(float NoV, float NoL, float roughness) {
    float a = roughness;
    float GGXV = NoL * (NoV * (1.0 - a) + a);
    float GGXL = NoV * (NoL * (1.0 - a) + a);
    return 0.5 / (GGXV + GGXL);
}

vec3 FillamentBRDF(in ShadingProperties props)
{
    vec3 F = FresnelSchlick(props.HoV, props.F0);
    float D = D_GGX(props.NoH, props.roughness);
    float V = V_SmithGGXCorrelated(props.NoV, props.NoL, props.roughness);

    vec3 specular = (D * V) * F;

    if (mulriScattering == 1) {
        vec3 energyCompensation = 1.0 + props.F0 * (1.0 / props.brdfLUT.x - 1.0);
        // Scale the specular lobe to account for multiscattering
        specular *= energyCompensation;
    }

    //Energy conservation
    vec3 kS = F;
    vec3 kD = (vec3(1.0) - kS) * (1.0 - props.metallic);

    return (kD * props.albedo.rgb * ONE_OVER_PI + specular) * lightColor.rgb * props.NoL;
}

vec3 UE4BRDF(in ShadingProperties props)
{
    vec3 F = FresnelSchlick(props.HoV, props.F0);
    float D = DistributionGGX(props.NoH, props.roughness);
    float V = V_SmithSchlickGGX(props.NoV, props.NoL, props.perceptualRoughness);

    vec3 numerator = (D * V) * F;

    float denominator = 4.0 * props.NoV * props.NoL;

    vec3 specular = numerator / max(denominator, EPSILON);

    //Energy conservation
    vec3 kS = F;
    vec3 kD = (vec3(1.0) - kS) * (1.0 - props.metallic);

    return (kD * props.albedo.rgb * ONE_OVER_PI + specular) * lightColor.rgb * props.NoL;
}

vec3 BRDF(in ShadingProperties props)
{
    switch (brdfType) {
        case BRDF_FILLAMENT:
            return FillamentBRDF(props);
        case BRDF_UE4:
            return UE4BRDF(props);
        default:
            return vec3(1.0, 0.0, 1.0); // Error
    }
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

vec3 IBL(in ShadingProperties props)
{
    vec3 F = FresnelSchlick(props.NoV, props.F0);

    vec3 kD = 1.0 - F;
    kD *= 1.0 - props.metallic;

    vec3 indirectDiffuse = props.irradiance * props.albedo.rgb;
    vec3 indirectSpecular = props.radiance * (props.F0 * props.brdfLUT.x + props.brdfLUT.y);

    if (specularAO == 1) {
        indirectSpecular *= props.so;
        indirectSpecular *= props.horizonSo;
        return kD * indirectDiffuse * props.ao + indirectSpecular;
    }

    return (kD * indirectDiffuse + indirectSpecular) * props.ao;
}

// reference: https://github.com/google/filament/blob/main/shaders/src/light_indirect.fs
vec3 GetSpecularDominantDirection(const vec3 n, const vec3 r, in float roughness)
{
    return mix(r, n, roughness * roughness);
}

// reference: https://github.com/google/filament/blob/main/shaders/src/light_indirect.fs
float PerceptualRoughnessToLod(in float perceptualRoughness)
{
    return maxReflectionLod * perceptualRoughness * (2.0 - perceptualRoughness);
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

void PopulateVectorProducts(inout ShadingProperties props)
{
    props.t = normalize(fsIn.wTangent.xyz);
    mat3 tangentToWorldMat = CreateTangentToWorldMatrix(normalize(fsIn.wNormal), props.t, fsIn.wTangent.w);

    props.n = normalize(tangentToWorldMat * SampleNormalMap(normalMap, fsIn.texcoord, 1.0));

    vec3 v = normalize(fsIn.wViewDirection);
    vec3 l = normalize(wLightDirection).xyz;
    vec3 h = normalize(l + v);
    props.r = reflect(-v, props.n);

    props.NoH = clamp(dot(props.n, h), 0.0, 1.0);

    props.NoV = clamp(abs(dot(props.n, v)), 0.0, 1.0);

    props.NoL = clamp(dot(props.n, l), 0.0, 1.0);
    props.HoV = clamp(dot(h, v), 0.0, 1.0);
}

void PopulateMaterialProperties(inout ShadingProperties props)
{
    props.albedo = texture(albedoMap, fsIn.texcoord) * vec4(baseColor.rgb, 1.0);

    vec3 m_r_ao = texture(m_r_aoMap, fsIn.texcoord).rgb;
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
            return vec4(fsIn.texcoord, 0.0, 1.0);
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