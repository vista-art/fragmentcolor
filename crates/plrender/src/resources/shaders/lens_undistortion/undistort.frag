#version 330 core

in vec2 UV;

out vec4 color;

uniform sampler2D textureSampler;

void main()
{   
    vec2 focalLength = vec2(438.568f, 437.699f);
    vec2 opticalCenter = vec2(667.724f, 500.059f);
    vec4 distortionCoefficients = vec4(-0.035109f, -0.002393f, 0.000335f, -0.000449f);

    const vec2 imageSize = vec2(1280.f, 960.f);

    vec2 opticalCenterUV = opticalCenter / imageSize;

    vec2 shiftedUVCoordinates = (UV - opticalCenterUV);

    //vec2 lensCoordinates = shiftedUVCoordinates / focalLength;
    vec2 lensCoordinates = (UV * imageSize - opticalCenter) / focalLength;


    float radiusSquared = dot(lensCoordinates, lensCoordinates);
    float radiusQuadrupled = radiusSquared * radiusSquared;

    float coefficientTerm = distortionCoefficients.x * radiusSquared + distortionCoefficients.y * radiusQuadrupled;

    vec2 distortedUV = ((lensCoordinates + lensCoordinates * (coefficientTerm))) * focalLength;

    vec2 resultUV = (distortedUV + opticalCenterUV);

    color = texture2D(textureSampler, resultUV);
}
