#version 460 core
layout(location = 0) in vec4 position;
uniform float test2;
void main()
{
    gl_Position = position;
}