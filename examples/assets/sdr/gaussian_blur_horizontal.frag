#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0, binding = 0) uniform sampler2D image;

const float weight[5] = float[] (0.2270270270, 0.1945945946, 0.1216216216, 0.0540540541, 0.0162162162);

in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

void main()
{
    vec2 texelSize = 1.0 / textureSize(image, 0);
    vec3 color = texture(image, fsIn.texcoord).rgb * weight[0];

    for(int i = 1; i < 5; ++i)
    {
        color += texture(image, fsIn.texcoord + vec2(texelSize.x * i, 0.0)).rgb * weight[i];
        color += texture(image, fsIn.texcoord - vec2(texelSize.x * i, 0.0)).rgb * weight[i];
    }

    outColor = vec4(color, 1.0);
}
