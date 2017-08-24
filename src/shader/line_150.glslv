#version 150 core

#define EPS 1E-6
#define uSize 0.012

in vec2 aStart, aEnd;
in uint aIdx;
out vec4 uvl;
out float vLen;


void main () {
    float tang;
    vec2 current;
    // All points in quad contain the same data:
    // segment start point and segment end point.
    // We determine point position from it's index.
    float idx = float(aIdx);
    if (aIdx >= 2u) {
        current = aEnd;
        tang = 1.0;
    } else {
        current = aStart;
        tang = -1.0;
    }
    float side = (mod(idx, 2.0)-0.5)*2.0;
    uvl.xy = vec2(tang, side);
    uvl.w = floor(idx / 4.0 + 0.5);

    vec2 dir = aEnd-aStart;
    uvl.z = length(dir);
    if (uvl.z > EPS) {
        dir = dir / uvl.z;
    } else {
    // If the segment is too short draw a square;
        dir = vec2(1.0, 0.0);
    }
    vec2 norm = vec2(-dir.y, dir.x);
    gl_Position = vec4((current+(tang*dir+norm*side)*uSize),0.0,1.0);
    //gl_PointSize = 20.0;
}