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

class ProgramInfo {
    async setUp(gl) {
        this.gl = gl;

        await this.loadProgram();
        this.fetchLocations();
    }

    async loadProgram() {
        const shaders = [
            this.loadShader(this.gl.VERTEX_SHADER, await sources.vertex),
            this.loadShader(this.gl.FRAGMENT_SHADER, await sources.fragment),
        ];

        this.program = this.gl.createProgram();
        for (const shader of shaders) {
            this.gl.attachShader(this.program, await shader);
        }
        this.gl.linkProgram(this.program);

        if (!this.gl.getProgramParameter(this.program, this.gl.LINK_STATUS)) {
            const log = this.gl.getProgramInfoLog(this.program);
            this.gl.deleteProgram(this.program);
            throw {
                shaders: shaders,
                log: log,
            };
        }
    }

    async loadShader(type, source) {
        const shader = this.gl.createShader(type);
        this.gl.shaderSource(shader, source);
        this.gl.compileShader(shader);

        if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
            const log = this.gl.getShaderInfoLog(shader);
            this.gl.deleteShader(shader);
            throw {
                type: type,
                source: source,
                log: log,
            };
        }

        return shader;
    }

    fetchLocations() {
        this.uniformLocations = {
            model: this.gl.getUniformLocation(this.program, "u_model"),
            view: this.gl.getUniformLocation(this.program, "u_view"),
        };

        this.attributeLocations = {
            position: this.gl.getAttribLocation(this.program, "a_position"),
            center: this.gl.getAttribLocation(this.program, "a_center"),
        };
    }

    bind(data) {
        this.gl.useProgram(this.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.model, false, data.matrices.model);
        this.gl.uniformMatrix4fv(this.uniformLocations.view, false, data.matrices.view);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, data.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, data.buffers.center);
        this.gl.vertexAttribPointer(
            this.attributeLocations.center,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.center);
    }
}

class ShaderData {
    async setUp(gl) {
        this.canvas = gl.canvas;
        this.initMatrices();
        await this.initBuffers(gl);
    }

    initMatrices() {
        this.matrices = {
            model: new Float32Array([
                1000.0, 0.0, 0.0, 0.0,
                0.0, 2000.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                -12500.0, -104540.0, 0.0, 1.0,
            ]),
            view: new Float32Array([
                2.0 / this.canvas.width, 0.0, 0.0, 0.0,
                0.0, 2.0 / this.canvas.height, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                -1.0, -1.0, 0.0, 1.0,
            ]),
        };
    }

    async initBuffers(gl) {
        this.buffers = {};

        const data = JSON.parse(await sources.data);
        this.length = 3 * data.length;

        let positions = [];
        let centers = [];

        for (let stop of data) {
            positions.push(
                stop.lon - 10.0, stop.lat - 10.0,
                stop.lon - 10.0, stop.lat + 30.0,
                stop.lon + 30.0, stop.lat - 10.0,
            );
            centers.push(
                stop.lon, stop.lat,
                stop.lon, stop.lat,
                stop.lon, stop.lat,
            );
        }

        this.buffers.position = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffers.position);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

        this.buffers.center = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffers.center);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(centers), gl.STATIC_DRAW);
    }
}

class Controller {
    async setUp(gl) {
        this.gl = gl;
        this.initializeCanvas();

        this.programInfo = new ProgramInfo();
        this.shaderData = new ShaderData();

        await Promise.all([
            this.programInfo.setUp(this.gl),
            this.shaderData.setUp(this.gl),
        ]);
        this.draw();
        addEventListener("resize", () => this.draw());
    }

    initializeCanvas() {
        this.resizeCanvas();
        addEventListener("resize", () => this.resizeCanvas());
    }

    resizeCanvas() {
        const canvas = this.gl.canvas;
        canvas.width = document.body.clientWidth;
        canvas.height = document.body.clientHeight;
        this.gl.viewport(0, 0, canvas.width, canvas.height);
        this.clear();
    }

    clear() {
        this.gl.clearColor(0.85, 0.9, 0.9, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }

    draw() {
        this.programInfo.bind(this.shaderData);

        this.gl.disable(this.gl.DEPTH_TEST);
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFuncSeparate(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA, this.gl.ZERO, this.gl.ONE);

        this.clear();
        this.gl.drawArrays(this.gl.TRIANGLES, 0, this.shaderData.length);
    }
}

const sources = {
    vertex: loadSource("shader.vert.glsl"),
    fragment: loadSource("shader.frag.glsl"),
    data: loadSource("../data/vbb.json"),
};

const controller = new Controller();

addEventListener("load", () => {
    const canvas = document.querySelector("canvas");
    const gl = canvas.getContext("webgl", {alpha: false});
    controller.setUp(gl).catch(error => console.error(error));
});
