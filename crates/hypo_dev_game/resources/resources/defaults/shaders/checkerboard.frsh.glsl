#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
layout(location = 3) out vec3 frag_emissive;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;

//https://www.shadertoy.com/view/tdBXRW
float xorTexture( in vec2 pos )
{
    float xor = 0.0;
    for( int i=0; i<8; i++ )
    {
        xor += mod( floor(pos.x)+floor(pos.y), 2.0 );

        pos *= 0.5;
        xor *= 0.5;
    }
    return xor;
}


void main() {
	float val = xorTexture(m_position.xz * 10.0);	
	frag_diffuse = vec3(val, val, val);
	frag_normal = m_normal;
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}