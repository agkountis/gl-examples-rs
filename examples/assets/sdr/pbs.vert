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

layout(location = 8) uniform vec3 eyePosition;

out gl_PerVertex {
    vec4 gl_Position;
};

// Varying variables
// prefixes: w -> world space
//           v -> view space
//           t -> tangent space
//           l -> local space
layout(location = 0) out VsOut {
    vec3 wLightDirection;
    vec3 wViewDirection;
    vec2 texcoord;
    mat3 TBN;
} vsOut;

void main()
{
    //Transform vertex to clipspace.
    vec4 lVertexPosition = vec4(inPosition, 1.0);
    vec4 wVertexPosition = model * lVertexPosition;
    gl_Position = projection * view * wVertexPosition;

    // Calculate the normal matrix.
    mat3 normalMatrix = mat3(transpose(inverse(model)));

    //Calculate the normal. Bring it to world space
    vec3 wNormal = normalMatrix * inNormal;

    // Bring tangent to world space.
    vec3 wTangent = normalize(normalMatrix * inTangent);

    wTangent = normalize(wTangent - dot(wTangent, wNormal) * wNormal);

    //Calculate the binormal
    vec3 wBinormal = normalize(cross(wNormal, wTangent));

    // Pass the TBN matrix to the fragment shader.
    vsOut.TBN = mat3(wTangent, wBinormal, wNormal);

    //Assign the view direction for output.
    vsOut.wViewDirection = eyePosition - wVertexPosition.xyz;

    //Assign texture coorinates for output.
    vsOut.texcoord = inTexcoord;
}
