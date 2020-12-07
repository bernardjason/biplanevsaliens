#version 330 core
out vec4 FragColor;

in vec2 TexCoord;
in vec3 Pos;

// texture samplers
uniform sampler2D texture0;
uniform vec3 wave;

void main()
{
	vec2 c = vec2(TexCoord.x,TexCoord.y);
	if ( TexCoord.x > 0.5 && TexCoord.x < 0.6 ) {
	    c.x = c.x + wave.y;
	    FragColor = texture(texture0, c);
	} else {
	    FragColor = texture(texture0, c);
	}

}
