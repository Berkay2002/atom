// Raw WebGL2 ray-march renderer.
//
// Mirrors the desktop wgpu renderer in `crates/atom-desktop/src/render.rs`
// in concept: upload an R32F 3D texture, draw a fullscreen triangle, sample
// the volume in a slab-clipped fixed-step accumulate.
//
// Public API mirrors the desktop module: `setVolume`, `setCamera`,
// `setParams`, `resize`, `draw`. No React imports, no DOM globals beyond
// the canvas passed in.

import type { ColormapStops } from '../colormaps';
import { buildColormapLutBytes } from './raymarch-contract';
import { FRAG_SRC, VERT_SRC } from './shaders';

export type RaymarchParams = {
  /** Absorption coefficient `k` in `1 - exp(-k * sum)`. */
  k: number;
  /** Output exposure multiplier. */
  exposure: number;
  /** Number of fixed-step ray-march samples through the volume. */
  steps: number;
};

export type VolumeData = {
  /** Length must be res³, row-major x→y→z (matches atom-core::volume::bake). */
  data: Float32Array;
  /** Cubic grid edge length (voxels per side). */
  res: number;
  /** Half-extent of the cubic bounding box, in Bohr radii. */
  halfExtent: number;
};

export type CameraState = {
  /** World-space camera position. */
  position: [number, number, number];
  /** Pre-inverted view-projection matrix (column-major 4x4, length 16). */
  invViewProj: Float32Array;
};

function compileShader(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type);
  if (!sh) throw new Error('createShader returned null');
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(sh) ?? '<no log>';
    gl.deleteShader(sh);
    throw new Error(`shader compile failed:\n${log}`);
  }
  return sh;
}

function linkProgram(gl: WebGL2RenderingContext, vs: WebGLShader, fs: WebGLShader): WebGLProgram {
  const prog = gl.createProgram();
  if (!prog) throw new Error('createProgram returned null');
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(prog) ?? '<no log>';
    gl.deleteProgram(prog);
    throw new Error(`program link failed:\n${log}`);
  }
  return prog;
}

export class Raymarcher {
  private readonly gl: WebGL2RenderingContext;
  private readonly program: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject;
  private readonly uniforms: {
    invViewProj: WebGLUniformLocation;
    camPos: WebGLUniformLocation;
    boxHalf: WebGLUniformLocation;
    params: WebGLUniformLocation;
    volume: WebGLUniformLocation;
    lut: WebGLUniformLocation;
  };
  private readonly linearFilteringSupported: boolean;
  // SRGB8_ALPHA8 is core in WebGL2; the legacy EXT_sRGB extension only
  // applies to WebGL1. If a future browser somehow drops the core format
  // we fall back to plain RGBA8 (linear), accepting slightly washed-out
  // colors vs the desktop's `Rgba8UnormSrgb` upload.
  private readonly srgbLutSupported = true;

  private texture: WebGLTexture | null = null;
  private lutTexture: WebGLTexture | null = null;
  private res = 0;
  private boxHalf = 1;
  private camPos: [number, number, number] = [0, 0, 5];
  private invViewProj: Float32Array = new Float32Array(16);
  private params: RaymarchParams = { k: 5, exposure: 1, steps: 256 };

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl2', {
      antialias: false,
      alpha: false,
      depth: false,
      preserveDrawingBuffer: false,
    });
    if (!gl) throw new Error('WebGL2 is not available in this browser');
    this.gl = gl;

    // R32F textures are core in WebGL2 but linear filtering on float
    // formats requires the OES_texture_float_linear extension. It's
    // present on virtually every desktop GPU; fall back to NEAREST
    // if a client lacks it.
    this.linearFilteringSupported =
      gl.getExtension('OES_texture_float_linear') !== null;

    const vs = compileShader(gl, gl.VERTEX_SHADER, VERT_SRC);
    const fs = compileShader(gl, gl.FRAGMENT_SHADER, FRAG_SRC);
    this.program = linkProgram(gl, vs, fs);
    gl.deleteShader(vs);
    gl.deleteShader(fs);

    const vao = gl.createVertexArray();
    if (!vao) throw new Error('createVertexArray returned null');
    this.vao = vao;

    const must = (name: string): WebGLUniformLocation => {
      const loc = gl.getUniformLocation(this.program, name);
      if (loc === null) throw new Error(`uniform ${name} not found`);
      return loc;
    };
    this.uniforms = {
      invViewProj: must('u_inv_view_proj'),
      camPos: must('u_cam_pos'),
      boxHalf: must('u_box_half'),
      params: must('u_params'),
      volume: must('u_volume'),
      lut: must('u_lut'),
    };

    gl.clearColor(0, 0, 0, 1);
  }

  /**
   * Upload a 256-entry RGBA8 1D look-up texture interpolated from the
   * caller's stop array. Mirrors `upload_lut_texture` in
   * `crates/atom-desktop/src/render.rs` — the interpolation math is byte-
   * identical so the web and desktop palettes match.
   *
   * WebGL2 has no 1D texture target, so the LUT lives in a 256×1 2D
   * texture. The fragment shader samples it at `(intensity, 0.5)`.
   *
   * Cheap enough that re-creating the storage on every call is fine —
   * 1 KiB upload + no pipeline rebuild keeps the swap snappy.
   */
  setColormap(stops: ColormapStops): void {
    const { gl } = this;
    const data = buildColormapLutBytes(stops);

    if (!this.lutTexture) {
      const tex = gl.createTexture();
      if (!tex) throw new Error('createTexture returned null for LUT');
      this.lutTexture = tex;
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    } else {
      gl.bindTexture(gl.TEXTURE_2D, this.lutTexture);
    }

    const internalFormat = this.srgbLutSupported ? gl.SRGB8_ALPHA8 : gl.RGBA8;
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 4);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      internalFormat,
      256,
      1,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      data,
    );
  }

  setVolume(vol: VolumeData): void {
    const { gl } = this;
    if (vol.data.length !== vol.res * vol.res * vol.res) {
      throw new Error(
        `volume data length ${vol.data.length} != res³ (${vol.res}³ = ${vol.res ** 3})`
      );
    }

    if (this.texture) gl.deleteTexture(this.texture);
    const tex = gl.createTexture();
    if (!tex) throw new Error('createTexture returned null');
    this.texture = tex;

    gl.bindTexture(gl.TEXTURE_3D, tex);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 4);
    gl.texImage3D(
      gl.TEXTURE_3D,
      0,
      gl.R32F,
      vol.res,
      vol.res,
      vol.res,
      0,
      gl.RED,
      gl.FLOAT,
      vol.data
    );

    const filter = this.linearFilteringSupported ? gl.LINEAR : gl.NEAREST;
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);

    this.res = vol.res;
    this.boxHalf = vol.halfExtent;
  }

  setCamera(cam: CameraState): void {
    this.camPos = cam.position;
    // Copy so callers can reuse their matrix buffer between frames.
    this.invViewProj.set(cam.invViewProj);
  }

  setParams(params: RaymarchParams): void {
    this.params = params;
  }

  resize(widthPx: number, heightPx: number): void {
    const { gl } = this;
    const w = Math.max(1, Math.floor(widthPx));
    const h = Math.max(1, Math.floor(heightPx));
    if (gl.canvas.width !== w) gl.canvas.width = w;
    if (gl.canvas.height !== h) gl.canvas.height = h;
    gl.viewport(0, 0, w, h);
  }

  draw(): void {
    const { gl } = this;
    if (!this.texture || this.res === 0) {
      // Nothing to draw yet; keep the canvas black.
      gl.clear(gl.COLOR_BUFFER_BIT);
      return;
    }

    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_3D, this.texture);
    gl.uniform1i(this.uniforms.volume, 0);

    if (this.lutTexture) {
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, this.lutTexture);
      gl.uniform1i(this.uniforms.lut, 1);
    }

    gl.uniformMatrix4fv(this.uniforms.invViewProj, false, this.invViewProj);
    gl.uniform3f(this.uniforms.camPos, this.camPos[0], this.camPos[1], this.camPos[2]);
    gl.uniform1f(this.uniforms.boxHalf, this.boxHalf);
    gl.uniform4f(
      this.uniforms.params,
      this.params.k,
      this.params.exposure,
      this.params.steps,
      0
    );

    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.bindVertexArray(null);
  }

  dispose(): void {
    const { gl } = this;
    if (this.texture) gl.deleteTexture(this.texture);
    if (this.lutTexture) gl.deleteTexture(this.lutTexture);
    gl.deleteProgram(this.program);
    gl.deleteVertexArray(this.vao);
  }
}
