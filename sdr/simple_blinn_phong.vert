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

layout(location = 0) out vec2 outTexcoord;

out gl_PerVertex {
    vec4 gl_Position;
};

void main()
{
    vec4 localVertexPosition = vec4(inPosition, 1.0);
    gl_Position = projection * view * model * localVertexPosition;

    outTexcoord = inTexcoord;
}
