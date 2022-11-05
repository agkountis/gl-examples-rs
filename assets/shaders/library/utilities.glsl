#ifndef UTILITIES_GLSL_
#define UTILITIES_GLSL_

const float INVERSE_GAMMA = 1.0 / 2.2;

vec3 Pow3(vec3 v, float power)
{
    return vec3(pow(v.x, power), pow(v.y, power), pow(v.z, power));
}

vec3 LinearToSrgb(vec3 linearColor)
{
    return Pow3(linearColor, INVERSE_GAMMA);
}

// Rec 709 lumincance coefficients
// https://en.wikipedia.org/wiki/Luma_(video)
float Luminance(vec3 color)
{
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

// CCIR 601 luma coefficients
// https://en.wikipedia.org/wiki/Luma_(video)
// https://en.wikipedia.org/wiki/Rec._601
float LumaRec601(vec3 color)
{
    return dot(LinearToSrgb(color), vec3(0.299, 0.587, 0.114));
}

float LumaRec709(vec3 color)
{
    return dot(LinearToSrgb(color), vec3(0.2126, 0.7152, 0.0722));
}


// numSamples is the number of samples combined to create the value of "col"
// When using dual filtering we use 1 samples per color input in this function.
float KarisAverage(vec3 col, float numSamples)
{
    // Formula is 1 / (1 + luma)
    float luma = LumaRec709(col) * (1.0 / numSamples);
//    float luma = Luminance(col) * (1.0 / numSamples);
    return 1.0 / (1.0 + luma);
}

#endif // UTILITIES_GLSL_