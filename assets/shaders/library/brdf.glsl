#ifndef BRDF_GLSL_
#define BRDF_GLSL_

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


    //TODO: This is wrong.
    if (mulriScattering == 1) {
        //TODO: This is wrong.
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
    #if defined(FEATURE_BRDF_FILLAMENT)
        return FillamentBRDF(props);
    #elif defined(FEATURE_BRDF_UE4)
        return UE4BRDF(props);
    #else
        return vec3(1.0, 0.0, 1.0);
    #endif
}

#endif // BRDF_GLSL_