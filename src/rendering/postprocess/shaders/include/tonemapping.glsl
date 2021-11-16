#ifndef TONEMAPPING_FUNCTIONS_GLSL
#define TONEMAPPING_FUNCTIONS_GLSL

layout(std140, binding = 3) uniform ToneMappingBlock
{
    int tonemappingOperator;
    float whiteThreshold;
    float exposure;
};

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
// Reference: https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
vec3 ACESFitted(vec3 color)
{
    color = color * ACESInputMat;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = color * ACESOutputMat;

    // Clamp to [0, 1]
    color = clamp(color, 0.0, 1.0);

    return color;
}


//simple luminance fit. Oversaturates brights
// Reference: https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 ACESFilm(vec3 x)
{
    // The input in the post has been pre-exposed.
    // To get the original ACES curve we have to multiply by 0.6
    // Reference: https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
    x *= 0.6;
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}

// Reference: https://www.shadertoy.com/view/lslGzl
vec3 Reinhard(vec3 color)
{
    return color / (1.0 + color / exposure);
}

// Reference: https://www.shadertoy.com/view/lslGzl
vec3 LumaBasedReinhard(vec3 color)
{
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    float toneMappedLuma = luma / (1.0 + luma);
    color *= toneMappedLuma / luma;
    return color;
}

// Reference: https://www.shadertoy.com/view/lslGzl
vec3 WhitePreservingLumaBasedReinhard(vec3 color)
{
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    float toneMappedLuma = luma * (1. + luma / (whiteThreshold * whiteThreshold)) / (1.0 + luma);
    color *= toneMappedLuma / luma;
    return color;
}

// Reference: https://www.shadertoy.com/view/lslGzl
// Reference: https://www.slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting
// Reference: https://www.slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting/53
vec3 Uncharted2(vec3 color)
{
    float A = 0.22; // Shoulder Strength
    float B = 0.30; // Linear Strength
    float C = 0.10; // Linear Angle
    float D = 0.20; // Toe Strength
    float E = 0.01; // Toe Numerator
    float F = 0.30; // Toe Denominator
    float W = 11.2; // Linear White Point
    color = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
    float white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
    color /= white;
    return color;
}

// Reference: https://twitter.com/RomBinDaHouse/status/460354166788202496
// Reference: https://www.shadertoy.com/view/lslGzl
vec3 RomBinDaHouse(vec3 color)
{
    return exp(-1.0 / (2.72 * color + 0.15));
}

#endif // TONEMAPPING_FUNCTIONS_GLSL