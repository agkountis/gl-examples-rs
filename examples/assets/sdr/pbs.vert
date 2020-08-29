#version 450 core
#extension GL_ARB_separate_shader_objects : enable

//Vertex attributes
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec4 inTangent;
layout(location = 3) in vec2 inTexcoord;
layout(location = 4) in vec3 inColor;

layout(std140, binding = 0) uniform PerFrameBlock
{
    mat4 view_projection;
    vec4 eyePosition;
};

layout(std140, binding = 1) uniform PerDrawBlock
{
    mat4 model;
    mat4 normalMatrix;
};

out gl_PerVertex {
    vec4 gl_Position;
};

// Varying variables
// prefixes: w -> world space
//           v -> view space
//           t -> tangent space
//           l -> local space
layout(location = 0) out VsOut {
    vec3 wViewDirection;
    vec3 wNormal;
    vec4 wTangent;
    vec2 texcoord;
} vsOut;

void main()
{
    //Transform vertex to clipspace.
    vec4 lVertexPosition = vec4(inPosition, 1.0);
    vec4 wVertexPosition = model * lVertexPosition;
    gl_Position = view_projection * wVertexPosition;

    mat3 normalMat = mat3(normalMatrix);
    //Calculate the normal. Bring it to world space
    vsOut.wNormal = normalMat * inNormal;

    // Bring tangent to world space.
    vsOut.wTangent = vec4(normalMat * inTangent.xyz, inTangent.w);

    //Assign the view direction for output.
    vsOut.wViewDirection = eyePosition.xyz - wVertexPosition.xyz;

    //Assign texture coorinates for output.
    vsOut.texcoord = inTexcoord;
}
