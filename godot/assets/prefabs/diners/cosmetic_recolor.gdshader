shader_type canvas_item;

// Color transformation parameters
instance uniform float hue_shift : hint_range(0.0, 360.0) = 0.0; // Hue shift in degrees
instance uniform float saturation : hint_range(0.0, 2.0) = 1.0;   // Saturation multiplier
instance uniform float value : hint_range(0.0, 2.0) = 1.0;        // Value/brightness multiplier
instance uniform float alpha : hint_range(0.0, 1.0) = 1.0;        // Transparency

// Convert RGB to HSV
vec3 rgb_to_hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// Convert HSV to RGB
vec3 hsv_to_rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void fragment() {
    // Sample the texture
    vec4 tex_color = texture(TEXTURE, UV);

    // Only process non-transparent pixels
    if (tex_color.a > 0.01) {
        // Convert to HSV
        vec3 hsv = rgb_to_hsv(tex_color.rgb);

        // Apply transformations
        // Hue shift (wraps around 0-1 range)
        hsv.x = fract(hsv.x + hue_shift / 360.0);

        // Saturation multiplier
        hsv.y = clamp(hsv.y * saturation, 0.0, 1.0);

        // Value/brightness multiplier
        hsv.z = clamp(hsv.z * value, 0.0, 1.0);

        // Convert back to RGB
        vec3 rgb = hsv_to_rgb(hsv);

        // Apply alpha
        COLOR = vec4(rgb, tex_color.a * alpha);
    } else {
        // Preserve fully transparent pixels
        COLOR = tex_color;
    }
}
