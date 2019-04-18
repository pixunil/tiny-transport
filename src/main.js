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
        deleteShader(shader);
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
        attributes: {
            position: gl.getAttribLocation(program, "a_position"),
        }
    };
}

function initBuffers(gl) {
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);

    const positions = [
        -0.5, -0.5,
        0.5, -0.5,
        0.5, 0.5,
    ];

    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

    return {
        position: positionBuffer,
    };
}

function clear(gl) {
    gl.clearColor(0.85, 0.9, 0.9, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
}

function draw(gl, programInfo, buffers) {
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
    gl.vertexAttribPointer(
        programInfo.attributes.position,
        2, gl.FLOAT,
        false, 0, 0);
    gl.enableVertexAttribArray(programInfo.attributes.position);

    gl.useProgram(programInfo.program);

    clear(gl);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
}

const sources = {
    vertex: loadSource("shader.vert.glsl"),
    fragment: loadSource("shader.frag.glsl"),
};

addEventListener("load", () => {
    const canvas = document.querySelector("canvas");
    const gl = canvas.getContext("webgl");
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
