#version 450 core
#extension GL_ARB_separate_shader_objects : enable

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out VsOut {
    vec2 texcoord;
} vsOut;

void main()
{
    vsOut.texcoord = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
    gl_Position = vec4(vsOut.texcoord * vec2( 2.0f, -2.0f ) + vec2( -1.0f, 1.0f), 0.0f, 1.0f);
    vsOut.texcoord.y = 1.0 - vsOut.texcoord.y;
}
