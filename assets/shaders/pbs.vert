#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/engine.glsl"

//Vertex attributes
INPUT(0, vec3, inPosition);
INPUT(1, vec3, inNormal);
INPUT(2, vec4, inTangent);
INPUT(3, vec2, inTexcoord);
INPUT(4, vec3, inColor);

UNIFORM_BLOCK_BEGIN(1, PerDrawBlock)
    mat4 model;
    mat4 normalMatrix;
UNIFORM_BLOCK_END

out gl_PerVertex {
    vec4 gl_Position;
};

// Varying variables
// prefixes: w -> world space
//           v -> view space
//           t -> tangent space
//           l -> local space
OUTPUT_BLOCK_BEGIN(0, VsOut)
    vec3 wViewDirection;
    vec3 wNormal;
    vec4 wTangent;
    vec2 texcoord;
OUTPUT_BLOCK_END_NAMED(vsOut)

void main()
{
    //Transform vertex to clipspace.
    vec4 lVertexPosition = vec4(inPosition, 1.0);
    vec4 wVertexPosition = model * lVertexPosition;
    gl_Position = LIB_VIEW_PROJECTION_MATRIX * wVertexPosition;

    mat3 normalMat = mat3(normalMatrix);
    //Calculate the normal. Bring it to world space
    vsOut.wNormal = normalMat * inNormal;

    // Bring tangent to world space.
    vsOut.wTangent = vec4(normalMat * inTangent.xyz, inTangent.w);

    //Assign the view direction for output.
    vsOut.wViewDirection = LIB_CAMERA_POSITION.xyz - wVertexPosition.xyz;

    //Assign texture coorinates for output.
    vsOut.texcoord = inTexcoord;
}
