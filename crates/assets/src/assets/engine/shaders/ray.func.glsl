/*
struct Intersection {
	float t;
	float hit;
	vec3 hitPoint;
	vec3 normal;
};

struct Plane {
	vec3 position;
	vec3 normal;
};

void intersectPlane(vec3 ray, Plane p, inout Intersection i) {
	float d = -dot(p.position, p.normal);
	float v = dot(ray, p.normal);
	float t = -(dot(cPos, p.normal) + d) / v;
	if(t > 0.0 && t < i.t){
		i.t = t;
		i.hit = 1.0;
		i.hitPoint = vec3(
			cPos.x + t * ray.x,
			cPos.y + t * ray.y,
			cPos.z + t * ray.z
		);
		i.normal = p.normal;
		float diff = clamp(dot(i.normal, lightDirection), 0.1, 1.0);
		float m = mod(i.hitPoint.x, 2.0);
		float n = mod(i.hitPoint.z, 2.0);
		if((m > 1.0 && n > 1.0) || (m < 1.0 && n < 1.0)){
			diff -= 0.5;
		}
		
		t = min(i.hitPoint.z, 100.0) * 0.01;
		i.color = vec3(diff + t);
	}
}
*/