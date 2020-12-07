#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
uniform vec3 spinning;

uniform sampler2D texture_diffuse1;

void main()
{
    FragColor = texture(texture_diffuse1, TexCoords);
    if ( TexCoords.x < 0.3 && TexCoords.y > 0.6 ) {
        if ( FragColor.x > 0.7 && FragColor.y > 0.7 && FragColor.z < 0.1  ) {
            if ( mod(spinning.x,6)  >= 3.0 ) {
                FragColor = vec4(0,0,0,1);
            } else {
                FragColor = vec4(0,0,0,0);
            }
        } else if ( FragColor.x > 0.7 ) {
                if ( mod(spinning.x,6)  <= 3 ) {
                        FragColor = vec4(0,0,0,1);
                } else {
                    FragColor = vec4(0,0,0,0);
                }
        } else {
                    FragColor = vec4(0,0,0,0);
        }
    }

}
