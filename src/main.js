function resizeDependent(callback) {
    callback();
    addEventListener("resize", callback);
}

function loadSource(url) {
    return new Promise((resolve, reject) => {
        let request = new XMLHttpRequest;
        request.open("get", url);
        request.onload = () => {
            if (200 <= request.status && request.status < 300) {
                resolve(request.response);
            } else {
                reject({
                    status: request.status,
                    text: request.statusText,
                });
            }
        };
        request.onerror = () => {
            reject({
                status: request.status,
                text: request.statusText,
            });
        };
        request.send();
    });
}

function resizeCanvas(canvas, gl) {
    canvas.width = document.body.clientWidth;
    canvas.height = document.body.clientHeight;
    gl.viewport(0, 0, canvas.width, canvas.height);
}

async function loadShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const log = gl.getShaderInfoLog(shader);
        gl.deleteShader(shader);
        throw {
            type: type,
            source: source,
            log: log,
        };
    }

    return shader;
}

async function initProgram(gl, shaders) {
    const program = gl.createProgram();
    for (const shader of shaders) {
        gl.attachShader(program, await shader);
    }
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        const log = gl.getProgramInfoLog(program);
        gl.deleteProgram(program);
        throw {
            shaders: shaders,
            log: log,
        };
    }

    return {
        program: program,
        uniforms: {
            model: gl.getUniformLocation(program, "u_model"),
        },
        attributes: {
            position: gl.getAttribLocation(program, "a_position"),
            center: gl.getAttribLocation(program, "a_center"),
        },
    };
}

function initBuffers(gl) {
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);

    const positions = [
        50.0, 50.0,
        50.0, 250.0,
        250.0, 50.0,
    ];

    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

    const centerBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, centerBuffer);

    const centers = [
        100.0, 100.0,
        100.0, 100.0,
        100.0, 100.0,
    ];

    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(centers), gl.STATIC_DRAW);

    return {
        position: positionBuffer,
        center: centerBuffer,
    };
}

function clear(gl) {
    gl.clearColor(0.85, 0.9, 0.9, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
}

function draw(gl, programInfo, buffers) {
    gl.useProgram(programInfo.program);

    const model = new Float32Array([
        2.0 / gl.canvas.width, 0.0, 0.0, 0.0,
        0.0, 2.0 / gl.canvas.height, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, -1.0, 0.0, 1.0,
    ]);
    gl.uniformMatrix4fv(programInfo.uniforms.model, false, model);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
    gl.vertexAttribPointer(
        programInfo.attributes.position,
        2, gl.FLOAT,
        false, 0, 0);
    gl.enableVertexAttribArray(programInfo.attributes.position);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.center);
    gl.vertexAttribPointer(
        programInfo.attributes.center,
        2, gl.FLOAT,
        false, 0, 0);
    gl.enableVertexAttribArray(programInfo.attributes.center);

    gl.disable(gl.DEPTH_TEST);
    gl.enable(gl.BLEND);
    gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ZERO, gl.ONE);

    clear(gl);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
}

const sources = {
    vertex: loadSource("shader.vert.glsl"),
    fragment: loadSource("shader.frag.glsl"),
};

addEventListener("load", () => {
    const canvas = document.querySelector("canvas");
    const gl = canvas.getContext("webgl", {alpha: false});
    resizeDependent(() => resizeCanvas(canvas, gl));
    clear(gl);

    initProgram(gl, [
        sources.vertex.then(source => loadShader(gl, gl.VERTEX_SHADER, source)),
        sources.fragment.then(source => loadShader(gl, gl.FRAGMENT_SHADER, source))
    ]).then(programInfo => {
        const buffers = initBuffers(gl);
        resizeDependent(() => draw(gl, programInfo, buffers));
    }).catch(error => {
        console.error(error);
    });
});
