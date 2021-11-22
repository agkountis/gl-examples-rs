#ifndef IBL_GLSL_
#define IBL_GLSL_

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

#endif // IBL_GLSL_