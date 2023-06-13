// literally copy pasted from https://www.shadertoy.com/view/llffzM
// and from https://www.shadertoy.com/view/Ml2cWG

const float pi = 3.14159265359;
const float invPi = 1.0 / pi;

const float zenithOffset = 0.0;
const float multiScatterPhase = 0.2;
const float density = 0.6;

const float anisotropicIntensity = 0.2; //Higher numbers result in more anisotropic scattering

const vec3 skyColor = vec3(0.39, 0.57, 1.0) * (1.0 + anisotropicIntensity); //Make sure one of the conponents is never 0.0

#define smooth(x) x*x*(3.0-2.0*x)
#define zenithDensity(x) density / pow(max(x - zenithOffset, 0.35e-2), 0.75)

float greatCircleDist(vec2 p, vec2 lp)
{
    float phi_1 = p.y;
    float phi_2 = lp.y;
    float delta_lambda = p.x-lp.x;
    return acos(sin(phi_1)*sin(phi_2) + cos(phi_1)*cos(phi_2)*cos(delta_lambda));
}

vec3 getSkyAbsorption(vec3 x, float y){
	
	vec3 absorption = x * -y;
	     absorption = exp2(absorption) * 2.0;
	
	return absorption;
}

float getSunPoint(vec2 p, vec2 lp){
    float dist = greatCircleDist(p, lp)/pi*2.;
	return smoothstep(0.03, 0.026, dist) * 50.0;
}

float getRayleigMultiplier(vec2 p, vec2 lp)
{
    float dist = greatCircleDist(p, lp)/pi*5.;
	return 1.0 + pow(1.0 - clamp(dist, 0.0, 1.0), 2.0) * pi * 0.5;
}

float getMie(vec2 p, vec2 lp){
    float dist = greatCircleDist(p, lp)/pi*2.;
	float disk = clamp(1.0 - pow(dist, 0.1), 0.0, 1.0);
	
	return disk*disk*(3.0 - 2.0 * disk) * 2.0 * pi;
}

vec3 getAtmosphericScattering(vec2 p, vec2 lp){		
	float zenith = zenithDensity(p.y);
	float sunPointDistMult =  clamp(length(max(lp.y + multiScatterPhase - zenithOffset, 0.0)), 0.0, 1.0);
	
	float rayleighMult = getRayleigMultiplier(p, lp);
	
	vec3 absorption = getSkyAbsorption(skyColor, zenith);
    vec3 sunAbsorption = getSkyAbsorption(skyColor, zenithDensity(lp.y + multiScatterPhase));
	vec3 sky = skyColor * zenith * rayleighMult;
	vec3 sun = getSunPoint(p, lp) * absorption;
	vec3 mie = getMie(p, lp) * sunAbsorption;
	
	vec3 totalSky = mix(sky * absorption, sky / (sky + 0.5), sunPointDistMult);
//         totalSky += sun + mie;
	     totalSky *= sunAbsorption * 0.5 + 0.5 * length(sunAbsorption);

    return totalSky;
}

vec2 screen2world(vec2 pos)
{
    return (pos - 0.5) * vec2(2., 1.) * pi;
}

// A bit of conversion magic from https://learnopengl.com/PBR/IBL/Diffuse-irradiance
const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 sample_spherical_map(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

// Calculate a procedural sky color based on a multitude of gradients
vec3 calculate_sky_color(
    vec3 normal,
    vec3 sun
) {
    /*
    //TODO: Implement cube map and diffuse IBL to re-enable this
    vec2 test_normal = sample_spherical_map(normal);
    vec2 test_sun = sample_spherical_map(-sun);
    return pow(getAtmosphericScattering(screen2world(test_normal), screen2world(test_sun)) / 3, vec3(2.2));
    */

    // Get up component of vector and remap to 0 - 1 range
    float up = normal.y * 0.5 + 0.5;

    // Define color mapping values
    const vec3 dark_blue = pow(vec3(0.137,0.263,0.463) * 0.5, vec3(2.2));
    const vec3 light_blue = pow(vec3(0.533,0.733,0.857), vec3(2.2));
    const vec3 orange = pow(vec3(247.0, 134.0, 64.0) / 255.0, vec3(2.2));

    // Do some color mapping (day color)
    vec3 day_color = mix(light_blue, dark_blue, max(up, 0.0));
    
    // Do some color mapping (sunset color)
    vec3 sunset_color = mix(orange, dark_blue, max(up, 0.2));

    // Mix in night color
    float time_of_day = min(max(-sun.y, 0), 0.25) * 4;
    vec3 color = mix(day_color, vec3(0.001), 1-time_of_day);

    /*
    // TODO: Readd when we have proper PBR ambient and specular lighting
    // Create a procedural sun with the scene params
	float sun_value = dot(normal, -sun);
	sun_value = pow(max(sun_value, 0), 1300) * 30;
    color += vec3(sun_value);
    */

    return color;
}