#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;
out vec3 use_colour;

uniform mat4 model;
uniform vec3 colour;

void main()
{
	gl_Position = model *  vec4(aPos, 1.0f);
	gl_Position = gl_Position * vec4(1,1,1,1);
	TexCoord = vec2(aTexCoord.x, aTexCoord.y);
	use_colour = colour;
}
