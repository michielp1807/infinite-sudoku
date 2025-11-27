// Geometry for WebGL
const VERTICES = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]); // (x,y) corners
const VERTEX_COORD_SIZE = 2; // only (x,y)
const NUMBER_OF_VERTICES = 4; // 4 vertices

/**
 * Show an error message
 * @param {string} message
 * @return {never} 
 */
function oopsie(message) {
    alert("Error: " + message);
    throw new Error(message);
}

/**
 * Setup WebGL canvas
 * @param {HTMLCanvasElement} canvas 
 */
export default async function glSetup(canvas) {
    // Get WebGL context
    const gl = canvas.getContext("webgl2", { antialias: true, premultipliedAlpha: false })
        ?? oopsie("Unable to initialize WebGL. Your browser may not support WebGL.");

    // Setup viewport
    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clearColor(1, 1, 1, 1);

    // Compile and attach shaders
    const gl_program = gl.createProgram() ?? oopsie("Could not create WebGL program");

    /**
     * Attach a shader to our gl_program
     * @param {string} path 
     * @param {number} type 
     * @returns 
     */
    async function loadShader(path, type) {
        const shader = gl.createShader(type) ?? oopsie("Could not create WebGL program");
        const response = await fetch(path);
        if (!response.ok) oopsie(`Could not load shader from "${path}"`);
        const source = await response.text();
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        gl.attachShader(gl_program, shader);
        return shader;
    }

    const [vs, fs] = await Promise.all([
        loadShader("./assets/shader.vert", gl.VERTEX_SHADER),
        loadShader("./assets/shader.frag", gl.FRAGMENT_SHADER)
    ]);

    gl.linkProgram(gl_program);

    // Delete shaders
    gl.detachShader(gl_program, vs);
    gl.detachShader(gl_program, fs);
    gl.deleteShader(vs);
    gl.deleteShader(fs);

    // Check if link was successful
    if (!gl.getProgramParameter(gl_program, gl.LINK_STATUS)) {
        oopsie(gl.getProgramInfoLog(gl_program) ?? "WebGL link status negative");
    }

    // Initialize buffers for attributes (geometry)
    const a_position = gl.getAttribLocation(gl_program, "a_position");
    gl.enableVertexAttribArray(a_position);
    const gl_buffer = gl.createBuffer() ?? oopsie("Could not create WebGL buffer");
    gl.bindBuffer(gl.ARRAY_BUFFER, gl_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, VERTICES, gl.STATIC_DRAW);
    gl.vertexAttribPointer(a_position, VERTEX_COORD_SIZE, gl.FLOAT, false, 0, 0);

    // Use the program
    gl.useProgram(gl_program);

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    let texture_index = 0;

    return {
        internal: gl,
        draw() {
            gl.clear(gl.COLOR_BUFFER_BIT);
            gl.drawArrays(gl.TRIANGLE_STRIP, 0, NUMBER_OF_VERTICES); // full screen rectangle
        },
        resize() {
            gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
        },
        /**
         * @typedef {("1f" | "1i" | "2fv" | "3fv" | "4fv")} UniformType
         */
        /**
         * @typedef {"1f" extends T ? number :
         *           "1i" extends T ? number :
         *           "2fv" extends T ? [number, number] | Float32Array :
         *           "3fv" extends T ? [number, number, number] | Float32Array :
         *           "4fv" extends T ? [number, number, number, number] | Float32Array :
         *           never } UniformValue<T>
         * @template {UniformType} T
         */
        /**
         * Get a shader uniform
         * @template {UniformType} T
         * @param {string} name 
         * @param {T} type 
         * @param {UniformValue<T>} initial_value 
         */
        uniform(name, type, initial_value) {
            const location = gl.getUniformLocation(gl_program, name)
                ?? oopsie(`Could not find uniform "${name}"`);

            // @ts-ignore
            const setter = gl["uniform" + type].bind(gl);
            let current_value = initial_value;
            /**@type {null | function(UniformValue<T>): void} */
            let on_change = null;
            const uniform = {
                /**
                 * @param {UniformValue<T>} value 
                 */
                set(value) {
                    current_value = value;
                    setter(location, value);
                    on_change?.(value);
                },
                /**
                 * @param {function(UniformValue<T>): UniformValue<T>} fun 
                 */
                setf(fun) {
                    current_value = fun(current_value);
                    setter(location, current_value);
                    on_change?.(current_value);
                },
                /**
                 * @return {UniformValue<T>} value 
                 */
                get() {
                    return current_value;
                },
                /**
                 * @param {function(UniformValue<T>)} fun 
                 */
                onchange(fun) {
                    on_change = fun;
                }
            }

            uniform.set(initial_value);
            return uniform;
        },
        /**
         * Get a texture
         * @param {string} sampler_name 
         * @param {number} filter
         */
        texture(sampler_name, filter = gl.LINEAR) {
            const index = texture_index;
            texture_index++;

            const texture = gl.createTexture();
            gl.activeTexture(gl.TEXTURE0 + index);
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, filter);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, filter);

            this.uniform(sampler_name, "1i", index);

            const channelsPerFormat = {
                [gl.RGBA]: 4,
                [gl.RGB]: 3,
                [gl.LUMINANCE_ALPHA]: 2,
                [gl.LUMINANCE]: 1,
            }

            return {
                /**
                 * Set the image source for the texture
                 * @param {TexImageSource} source 
                 * @param {keyof channelsPerFormat} format
                 */
                setSource(source, level = 0, format = gl.RGBA) {
                    gl.activeTexture(gl.TEXTURE0 + index);
                    gl.texImage2D(gl.TEXTURE_2D, level, format, format, gl.UNSIGNED_BYTE, source);
                },
                /**
                 * @param {Uint8Array} data
                 * @param {number} width
                 * @param {number} height
                 * @param {keyof channelsPerFormat} format
                 */
                setSourceArray(data, width, height, format = gl.RGBA) {
                    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1); // otherwise WebGL expects rows to be multiples of 4 bytes
                    gl.activeTexture(gl.TEXTURE0 + index);
                    gl.texImage2D(gl.TEXTURE_2D, 0, format, width, height, 0, format, gl.UNSIGNED_BYTE, data);
                },
                /**
                 * Load an image to use as source for the texture
                 * @param {string} src
                 * @return {Promise<void>}
                 */
                loadImage(src, level = 0) {
                    return new Promise((resolve) => {
                        const image = new Image();
                        image.onload = () => {
                            this.setSource(image, level);
                            resolve();
                        };
                        image.src = src;
                    });

                },
                activateMipmap() {
                    gl.activeTexture(gl.TEXTURE0 + index);
                    gl.generateMipmap(gl.TEXTURE_2D);
                    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);

                },
                /**
                 * Set a pixel on the texture
                 * @param {GLint} x 
                 * @param {GLint} y 
                 * @param {[number]} color 
                 */
                setPixel(x, y, color) {
                    const colorArray = new Uint8Array(color);
                    gl.activeTexture(gl.TEXTURE0 + index);
                    gl.texSubImage2D(gl.TEXTURE_2D, 0, x, y, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, colorArray);
                }
            };
        }
    }
}