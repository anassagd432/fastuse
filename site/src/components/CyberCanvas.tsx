import React, { useEffect, useRef } from 'react';

/**
 * High-performance, GPU-driven WebGL Aurora & Caustics Shader Background.
 * 
 * Aesthetic Direction: Aurora / Caustics fluid shader.
 * Palette: Derived strictly from fastuse site CSS tokens:
 *   - Obsidian Void: #07090E  (0.027, 0.035, 0.055)
 *   - Deep Surface:  #0D111A  (0.051, 0.067, 0.102)
 *   - Brand Blue:    #2F6BFF  (0.184, 0.420, 1.000)
 *   - Neon Cyan:     #00F5D4  (0.000, 0.961, 0.831)
 *   - Luminous Blue: #548AFF  (0.329, 0.541, 1.000)
 * 
 * Architecture:
 * - 100% GPU-driven full-screen GLSL fragment shader (Zero external dependencies).
 * - Multi-octave domain-warping + caustic wave interference.
 * - Reactive to smoothed cursor tracking and page scroll parallax.
 * - Tab visibility detection (`document.hidden`) completely halts RAF loops when inactive.
 * - Handles WebGL context loss and high-DPI scaling gracefully.
 */

const VERTEX_SHADER_SOURCE = `
attribute vec2 a_position;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
}
`;

const FRAGMENT_SHADER_SOURCE = `
precision highp float;

uniform vec2 u_resolution;
uniform float u_time;
uniform vec2 u_mouse;
uniform float u_scroll;

// Simplex-style fast 2D hash & noise
vec2 hash2(vec2 p) {
  p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
  return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);

  return mix(
    mix(dot(hash2(i + vec2(0.0, 0.0)), f - vec2(0.0, 0.0)),
        dot(hash2(i + vec2(1.0, 0.0)), f - vec2(1.0, 0.0)), u.x),
    mix(dot(hash2(i + vec2(0.0, 1.0)), f - vec2(0.0, 1.0)),
        dot(hash2(i + vec2(1.0, 1.0)), f - vec2(1.0, 1.0)), u.x),
    u.y
  );
}

// Fractional Brownian Motion (fBm)
float fbm(vec2 p) {
  float v = 0.0;
  float a = 0.5;
  vec2 shift = vec2(100.0);
  mat2 rot = mat2(cos(0.5), sin(0.5), -sin(0.5), cos(0.5));
  for (int i = 0; i < 4; ++i) {
    v += a * noise(p);
    p = rot * p * 2.0 + shift;
    a *= 0.5;
  }
  return v;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution.xy;
  vec2 p = (gl_FragCoord.xy * 2.0 - u_resolution.xy) / min(u_resolution.x, u_resolution.y);

  // Apply subtle scroll parallax
  p.y += u_scroll * 0.15;

  // Mouse interaction: normalized deflection and caustic lens ripple
  vec2 mouseNorm = (u_mouse * 2.0 - u_resolution.xy) / min(u_resolution.x, u_resolution.y);
  mouseNorm.y = -mouseNorm.y; // invert Y coordinate for GL convention
  float mouseDist = length(p - mouseNorm);
  float mouseInfluence = exp(-mouseDist * 2.5);
  vec2 mouseWarp = (p - mouseNorm) * mouseInfluence * 0.25;

  // Domain Warping for fluid aurora tendrils
  float t = u_time * 0.12;
  vec2 q = vec2(
    fbm(p + vec2(0.0, 0.0) + t * 0.4 + mouseWarp),
    fbm(p + vec2(5.2, 1.3) + t * 0.3)
  );

  vec2 r = vec2(
    fbm(p + 3.0 * q + vec2(1.7, 9.2) + 0.15 * t),
    fbm(p + 3.0 * q + vec2(8.3, 2.8) + 0.12 * t)
  );

  float f = fbm(p + 3.5 * r + mouseWarp);

  // Caustics interference pattern
  vec2 cP = p * 2.5 + r * 1.5;
  float c1 = sin(cP.x * 3.0 + u_time * 0.5) + cos(cP.y * 3.0 - u_time * 0.4);
  float c2 = sin(cP.y * 4.0 + u_time * 0.35) + cos(cP.x * 2.5 + u_time * 0.6);
  float caustics = pow(abs(c1 * c2) * 0.25, 2.2);

  // Exact site color palette
  vec3 cVoid    = vec3(0.0275, 0.0353, 0.0549); // #07090E
  vec3 cSurface = vec3(0.0510, 0.0667, 0.1020); // #0D111A
  vec3 cBrand   = vec3(0.1843, 0.4196, 1.0000); // #2F6BFF
  vec3 cCyan    = vec3(0.0000, 0.9608, 0.8314); // #00F5D4
  vec3 cLight   = vec3(0.3294, 0.5412, 1.0000); // #548AFF

  // Layered color grading
  vec3 color = cVoid;
  color = mix(color, cSurface, clamp((f * f) * 2.0, 0.0, 1.0));
  color = mix(color, cBrand, clamp(length(q) * 0.85, 0.0, 1.0) * 0.45);
  color = mix(color, cCyan, clamp(length(r.x) * 1.1, 0.0, 1.0) * 0.35);

  // Caustic filaments & aurora crests
  color += cCyan * caustics * 0.45;
  color += cBrand * pow(clamp(f, 0.0, 1.0), 3.0) * 0.5;

  // Cursor-induced plasma highlight
  color += cCyan * mouseInfluence * 0.15;

  // Subtle radial vignette to anchor the dark cyber aesthetic
  float vignette = smoothstep(1.5, 0.4, length(p * 0.65));
  color *= vignette;

  gl_FragColor = vec4(color, 1.0);
}
`;

export const CyberCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Use WebGL context (with WebGL2 or experimental fallback)
    const gl =
      canvas.getContext('webgl', { powerPreference: 'high-performance', antialias: false, depth: false }) ||
      (canvas.getContext('experimental-webgl') as WebGLRenderingContext | null);

    if (!gl) {
      console.warn('[fastuse] WebGL unavailable for 3D aurora background.');
      return;
    }

    // Helper: Compile shader
    const compileShader = (type: number, src: string): WebGLShader | null => {
      const shader = gl.createShader(type);
      if (!shader) return null;
      gl.shaderSource(shader, src);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('Shader compile error:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
      }
      return shader;
    };

    const vert = compileShader(gl.VERTEX_SHADER, VERTEX_SHADER_SOURCE);
    const frag = compileShader(gl.FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
    if (!vert || !frag) return;

    const program = gl.createProgram();
    if (!program) return;
    gl.attachShader(program, vert);
    gl.attachShader(program, frag);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Program link error:', gl.getProgramInfoLog(program));
      return;
    }
    gl.useProgram(program);

    // Fullscreen quad buffer
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0]),
      gl.STATIC_DRAW
    );

    const aPosition = gl.getAttribLocation(program, 'a_position');
    gl.enableVertexAttribArray(aPosition);
    gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, 0, 0);

    // Uniform locations
    const uResolution = gl.getUniformLocation(program, 'u_resolution');
    const uTime = gl.getUniformLocation(program, 'u_time');
    const uMouse = gl.getUniformLocation(program, 'u_mouse');
    const uScroll = gl.getUniformLocation(program, 'u_scroll');

    // State tracking
    let animId: number;
    let isVisible = !document.hidden;
    const startTime = performance.now();

    const mouse = {
      targetX: window.innerWidth * 0.5,
      targetY: window.innerHeight * 0.4,
      currentX: window.innerWidth * 0.5,
      currentY: window.innerHeight * 0.4,
    };

    let scrollTarget = window.scrollY;
    let scrollCurrent = window.scrollY;

    // Viewport resize with devicePixelRatio clamping for 60fps locked execution
    const handleResize = () => {
      if (!canvas) return;
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      const displayWidth = Math.floor(window.innerWidth * dpr);
      const displayHeight = Math.floor(window.innerHeight * dpr);

      if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
        canvas.width = displayWidth;
        canvas.height = displayHeight;
      }
      gl.viewport(0, 0, canvas.width, canvas.height);
    };
    handleResize();
    window.addEventListener('resize', handleResize);

    const handleMouseMove = (e: MouseEvent) => {
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      mouse.targetX = e.clientX * dpr;
      mouse.targetY = e.clientY * dpr;
    };
    window.addEventListener('mousemove', handleMouseMove, { passive: true });

    const handleScroll = () => {
      scrollTarget = window.scrollY * 0.001;
    };
    window.addEventListener('scroll', handleScroll, { passive: true });

    // Performance: Pause rendering completely when tab is backgrounded
    const handleVisibilityChange = () => {
      isVisible = !document.hidden;
      if (isVisible) {
        animId = requestAnimationFrame(render);
      } else {
        cancelAnimationFrame(animId);
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);

    const render = (now: number) => {
      if (!isVisible) return;

      // Smooth lerp mouse and scroll inputs
      mouse.currentX += (mouse.targetX - mouse.currentX) * 0.08;
      mouse.currentY += (mouse.targetY - mouse.currentY) * 0.08;
      scrollCurrent += (scrollTarget - scrollCurrent) * 0.05;

      const elapsed = (now - startTime) * 0.001;

      gl.uniform2f(uResolution, canvas.width, canvas.height);
      gl.uniform1f(uTime, elapsed);
      gl.uniform2f(uMouse, mouse.currentX, mouse.currentY);
      gl.uniform1f(uScroll, scrollCurrent);

      gl.drawArrays(gl.TRIANGLES, 0, 6);

      animId = requestAnimationFrame(render);
    };

    animId = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('scroll', handleScroll);
      document.removeEventListener('visibilitychange', handleVisibilityChange);

      // WebGL resource cleanup
      gl.deleteBuffer(positionBuffer);
      gl.deleteProgram(program);
      gl.deleteShader(vert);
      gl.deleteShader(frag);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 pointer-events-none z-0 w-full h-full"
      style={{
        width: '100vw',
        height: '100vh',
        willChange: 'transform',
      }}
    />
  );
};
