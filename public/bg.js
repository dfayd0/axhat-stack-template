// WebGL2 background noise shader
(function() {
    const canvas = document.getElementById('bg-canvas');
    if (!canvas) return;

    const gl = canvas.getContext('webgl2');
    if (!gl) {
        console.warn('WebGL2 not available, using solid background');
        return;
    }

    function resize() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        gl.viewport(0, 0, canvas.width, canvas.height);
    }
    resize();
    window.addEventListener('resize', resize);

    const vsSource = `#version 300 es
        void main() {
            vec2 positions[3] = vec2[3](
                vec2(-1.0, -1.0),
                vec2( 3.0, -1.0),
                vec2(-1.0,  3.0)
            );
            gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
        }
    `;

    const fsSource = `#version 300 es
        precision highp float;
        uniform vec2 u_resolution;
        uniform float u_time;
        out vec4 fragColor;

        float hash(vec2 p) {
            vec3 p3 = fract(vec3(p.x, p.y, p.x) * 0.13);
            p3 += dot(p3, p3.yzx + 3.333);
            return fract((p3.x + p3.y) * p3.z);
        }

        float noise(vec2 p) {
            vec2 i = floor(p);
            vec2 f = fract(p);
            vec2 u = f * f * (3.0 - 2.0 * f);
            float a = hash(i);
            float b = hash(i + vec2(1.0, 0.0));
            float c = hash(i + vec2(0.0, 1.0));
            float d = hash(i + vec2(1.0, 1.0));
            return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
        }

        float fbm(vec2 p) {
            float value = 0.0;
            float amplitude = 0.5;
            float frequency = 1.0;
            for (int i = 0; i < 5; i++) {
                value += amplitude * noise(p * frequency);
                frequency *= 2.0;
                amplitude *= 0.5;
                p += vec2(1.7, 9.2);
            }
            return value;
        }

        void main() {
            float aspect = u_resolution.x / u_resolution.y;
            vec2 uv = gl_FragCoord.xy / u_resolution.y;
            float t = u_time * 0.08;

            float n1 = fbm(uv * 3.0 + vec2(t * 0.3, t * 0.2));
            float n2 = fbm(uv * 5.0 - vec2(t * 0.2, t * 0.15) + vec2(n1 * 0.5));
            float n3 = fbm(uv * 2.0 + vec2(n2 * 0.3, n1 * 0.3) + vec2(t * 0.1));

            float pattern = n1 * 0.4 + n2 * 0.35 + n3 * 0.25;

            // Dark flowing shapes visible against neutral-950
            float base = 0.02;
            float variation = pattern * 0.12;

            // Add subtle brighter wisps
            float wisp = smoothstep(0.55, 0.7, n2) * 0.08;

            vec3 color = vec3(base + variation + wisp);
            fragColor = vec4(color, 1.0);
        }
    `;

    function compile(type, source) {
        const shader = gl.createShader(type);
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            console.error('Shader compile error:', gl.getShaderInfoLog(shader));
            gl.deleteShader(shader);
            return null;
        }
        return shader;
    }

    const vs = compile(gl.VERTEX_SHADER, vsSource);
    const fs = compile(gl.FRAGMENT_SHADER, fsSource);
    if (!vs || !fs) return;

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error('Program link error:', gl.getProgramInfoLog(program));
        return;
    }

    gl.useProgram(program);
    const uResolution = gl.getUniformLocation(program, 'u_resolution');
    const uTime = gl.getUniformLocation(program, 'u_time');

    // Empty VAO required for WebGL2
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    let startTime = performance.now() / 1000.0;

    function render() {
        const t = performance.now() / 1000.0 - startTime;
        gl.uniform2f(uResolution, canvas.width, canvas.height);
        gl.uniform1f(uTime, t);
        gl.drawArrays(gl.TRIANGLES, 0, 3);
        requestAnimationFrame(render);
    }

    console.log('bg-shader: running (WebGL2)');
    requestAnimationFrame(render);
})();
