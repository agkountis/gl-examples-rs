#ifndef PARALLAX_MAPPING_GLSL_
#define PARALLAX_MAPPING_GLSL_

// Reference: https://learnopengl.com/Advanced-Lighting/Parallax-Mapping
vec2 ParallaxMapping(vec2 texcoords, vec3 viewDirection)
{
    float displacement = texture(displacementMap, texcoords).r;

    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDirection.xy / max(viewDirection.z, EPSILON) * displacement * pomDisplacementScale;

    return texcoords - P;
}

// Reference: https://learnopengl.com/Advanced-Lighting/Parallax-Mapping
vec2 ParallaxMappingOffsetLimiting(vec2 texcoords, vec3 viewDirection)
{
    float displacement = texture(displacementMap, texcoords).r;

    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDirection.xy * displacement * pomDisplacementScale;

    return texcoords - P;
}

// Reference: https://learnopengl.com/Advanced-Lighting/Parallax-Mapping
vec2 SteepParallaxMapping(vec2 texcoords, vec3 viewDirection)
{
    // Calculate how many layers to use based on the angle of the Z axis in tangent space (points upwards)
    // and the view vector.
    float numLayers = mix(pomMaxLayers, pomMinLayers, abs(dot(vec3(0.0, 0.0, 1.0), viewDirection)));

    // calculate the size of each layer
    float layerDepth = 1.0 / numLayers;

    // depth of current layer
    float currentLayerDepth = 0.0;

    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDirection.xy / max(viewDirection.z, EPSILON) * pomDisplacementScale;
    vec2 deltaTexCoords = P / numLayers;

    vec2 currentTexCoords = texcoords;
    float currentDepthValue = texture(displacementMap, currentTexCoords).r;

    while (currentLayerDepth < currentDepthValue)
    {
        // shift texture coordinates along direction of P
        currentTexCoords -= deltaTexCoords;
        // get depthmap value at current texture coordinates
        currentDepthValue = texture(displacementMap, currentTexCoords).r;
        // get depth of next layer
        currentLayerDepth += layerDepth;
    }

    return currentTexCoords;
}

// Reference: https://learnopengl.com/Advanced-Lighting/Parallax-Mapping
vec2 ParallaxOcclusionMapping(vec2 texcoords, vec3 viewDirection)
{
    // Calculate how many layers to use based on the angle of the Z axis in tangent space (points upwards)
    // and the view vector.
    float numLayers = mix(pomMaxLayers, pomMinLayers, abs(dot(vec3(0.0, 0.0, 1.0), viewDirection)));

    // calculate the size of each layer
    float layerDepth = 1.0 / numLayers;

    // depth of current layer
    float currentLayerDepth = 0.0;

    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDirection.xy / max(viewDirection.z, EPSILON) * pomDisplacementScale;
    vec2 deltaTexCoords = P / numLayers;

    vec2 currentTexCoords = texcoords;
    float currentDepthValue = texture(displacementMap, currentTexCoords).r;

    while (currentLayerDepth < currentDepthValue)
    {
        // shift texture coordinates along direction of P
        currentTexCoords -= deltaTexCoords;
        // get depthmap value at current texture coordinates
        currentDepthValue = texture(displacementMap, currentTexCoords).r;
        // get depth of next layer
        currentLayerDepth += layerDepth;
    }

    // get texture coordinates before collision (reverse operations)
    vec2 prevTexCoords = currentTexCoords + deltaTexCoords;

    // get depth after and before collision for linear interpolation
    float afterDepth = currentDepthValue - currentLayerDepth;
    float beforeDepth = texture(displacementMap, prevTexCoords).r - currentLayerDepth + layerDepth;

    // interpolation of texture coordinates
    float weight = clamp(afterDepth / (afterDepth - beforeDepth), 0.0, 1.0);
    vec2 finalTexCoords = prevTexCoords * weight + currentTexCoords * (1.0 - weight);

    return finalTexCoords;
}

#endif // PARALLAX_MAPPING_GLSL_