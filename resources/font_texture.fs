#version 330 core
out vec4 FragColor;

in vec2 TexCoord;
in vec3 use_colour;

// texture samplers
uniform sampler2D texture0;

void main()
{
	vec4 t = texture(texture0, TexCoord) ;
	if (t.x > 0.0 ) {
	    t = vec4(use_colour.x,use_colour.y,use_colour.z,1.0);
	}
	FragColor = t;
}