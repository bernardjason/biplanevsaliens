#version 330 core
out vec4 FragColor;

in vec2 TexCoord;
in vec3 Pos;

// texture samplers
uniform sampler2D texture1;
uniform sampler2D texture2;

void main()
{
	// linearly interpolate between both textures (80% container, 20% awesomeface)
	//FragColor = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
	vec3 colour = vec3(1.0,1.0,1.0);
    if ( Pos.x == -0.5 ) { colour = vec3(0.0,1.0,0.0); } // green
    if ( Pos.x == 0.0 )  { colour = vec3(0.0,1.0,0.5); } // green
    if ( Pos.x == 0.5 )  { colour = vec3(0.0,0.0,1.0); } // blue
    if ( Pos.y == 0.5 )  { colour = vec3(1.0,0.0,0.0); } // red
    if ( Pos.y == 0.0 )  { colour = vec3(1.0,0.0,0.5); } // red
    if ( Pos.y == -0.5 ) { colour = vec3(1.0,0.0,1.0); } //  cyan
    if ( Pos.z == 0.5 )  { colour = vec3(1.0,1.0,0.0); } // yellow
    if ( Pos.z == 0.0 )  { colour = vec3(1.0,1.0,0.5); } // yellow
    if ( Pos.z == -0.5 ) { colour = vec3(0.0,1.0,1.0); }
	FragColor = vec4(colour.x,colour.y,colour.z,1);

}
