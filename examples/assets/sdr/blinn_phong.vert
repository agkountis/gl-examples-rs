#version 450 core
#extension GL_ARB_separate_shader_objects : enable

//Vertex attributes
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inTangent;
layout(location = 3) in vec2 inTexcoord;
layout(location = 4) in vec3 inColor;

//TODO: Use a UBO here
layout(location = 5) uniform mat4 model;
layout(location = 6) uniform mat4 view;
layout(location = 7) uniform mat4 projection;

out gl_PerVertex {
    vec4 gl_Position;
};

// Varying variables
// prefixes: m_ -> model space
//           v_ -> view space
//           t_ -> tangent space
//layout(location = 0) out vec3 t_OutlightDirection;
//layout(location = 1) out vec3 t_OutViewDirection;
//layout(location = 2) out vec2 outTexcoord;

layout(location = 0) out VsOut {
    vec3 tLightDirection;
    vec3 tViewDirection;
    vec2 texcoord;
} vsOut;

void main()
{
    //Transform vertex to clipspace.
    vec4 localVertexPosition = vec4(inPosition, 1.0);
    gl_Position = projection * view * model * localVertexPosition;

    mat3 normalMatrix = transpose(inverse(mat3(view * model)));

    //Calculate the normal. Bring it to view space
    vec3 normal = normalMatrix * inNormal;

    // Bring tangent to view space.
    vec3 tangent = normalize(normalMatrix * inTangent);

    tangent = normalize(tangent - dot(tangent, normal) * normal);

    //Calculate the binormal
    vec3 binormal = normalize(cross(normal, tangent));

    // TBN originally transforms from tangent space to world space
    // Inversing it will cause it to transform from whatever space the
    // normals/binormals/tangets are in to tangent space.
    // Note: TBN is orthogonal, so transposing it is equivalent to inverting it.
    mat3 TBN = transpose(mat3(tangent, binormal, normal));

    //Move the vertex in view space.
    vec3 v_vertexPosition = (view * model * localVertexPosition).xyz;

    //Assign the view direction for output.
    //    t_OutViewDirection = TBN * -v_vertexPosition;
    vsOut.tViewDirection = TBN * -v_vertexPosition;

    vec3 v_lightPosition = (view * vec4(0.0, 0.0, 1.0, 1.0)).xyz;

    //Calculate and assign the light direction for output.
    //    t_OutlightDirection = TBN * v_lightPosition;
    vsOut.tLightDirection = TBN * v_lightPosition;

    //Assign texture coorinates for output.
    vsOut.texcoord = inTexcoord;
}
