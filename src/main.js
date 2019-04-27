const vec2 = glMatrix.vec2;
const mat2d = glMatrix.mat2d;
const mat4 = glMatrix.mat4;

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
    async setUp(gl, sources) {
        this.gl = gl;

        await this.loadProgram(sources);
        this.fetchLocations();
    }

    async loadProgram(sources) {
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
        this.uniformLocations = {};
        const uniformCount = this.gl.getProgramParameter(this.program, this.gl.ACTIVE_UNIFORMS);
        for (let uniformIndex = 0; uniformIndex < uniformCount; uniformIndex++) {
            const uniform = this.gl.getActiveUniform(this.program, uniformIndex);
            const name = uniform.name.substr(2);
            this.uniformLocations[name] = this.gl.getUniformLocation(this.program, uniform.name);
        }

        this.attributeLocations = {};
        const attributeCount = this.gl.getProgramParameter(this.program, this.gl.ACTIVE_ATTRIBUTES);
        for (let attributeIndex = 0; attributeIndex < attributeCount; attributeIndex++) {
            const attribute = this.gl.getActiveAttrib(this.program, attributeIndex);
            const name = attribute.name.substr(2);
            this.attributeLocations[name] = this.gl.getAttribLocation(this.program, attribute.name);
        }
    }
}

class StationProgramInfo extends ProgramInfo {
    run(data) {
        this.gl.useProgram(this.program);

        this.gl.uniform1f(this.uniformLocations.size, 20.0);
        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, data.matrices.modelView);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, data.buffers.stationPosition);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.drawArrays(this.gl.POINTS, 0, data.sizes.station);
    }
}

class LineProgramInfo extends ProgramInfo {
    run(data) {
        this.gl.useProgram(this.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, data.matrices.modelView);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, data.buffers.linePosition);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, data.buffers.lineColor);
        this.gl.vertexAttribPointer(
            this.attributeLocations.color,
            3, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.color);

        this.gl.drawArrays(this.gl.TRIANGLES, 0, data.sizes.line);
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
            model: glMatrix.mat2d.create(),
            view: glMatrix.mat2d.create(),
            modelView: glMatrix.mat4.create(),
        };

        mat2d.scale(this.matrices.model, this.matrices.model, vec2.fromValues(2000.0, 4000.0));
        mat2d.translate(this.matrices.model, this.matrices.model, vec2.fromValues(-13.5, -52.53));

        this.calculateModelView();
    }

    async initBuffers(gl) {
        this.buffers = {
            linePosition: gl.createBuffer(),
            lineColor: gl.createBuffer(),
            stationPosition: gl.createBuffer(),
        };
        this.sizes = {};

        const data = JSON.parse(await sources.data);

        let stationPositions = [];
        for (let id in data.locations) {
            const location = data.locations[id];
            stationPositions.push(location.lon, location.lat);
        }

        this.sizes.station = stationPositions.length / 2;
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffers.stationPosition);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(stationPositions), gl.STATIC_DRAW);

        let linePositions = [];
        let lineColors = [];
        for (let route of data.routes) {
            const color = route.color.map(component => component / 255);
            for (let i = 0; i < route.stops.length - 1; i++) {
                let curr = data.locations[route.stops[i]];
                let next = data.locations[route.stops[i + 1]];
                curr = vec2.fromValues(curr.lon, curr.lat);
                next = vec2.fromValues(next.lon, next.lat);

                let direction = vec2.subtract(vec2.create(), curr, next);
                direction[1] *= 2;
                vec2.normalize(direction, direction);

                linePositions.push(
                    curr[0] + 0.004 * direction[1], curr[1] - 0.002 * direction[0],
                    curr[0] - 0.004 * direction[1], curr[1] + 0.002 * direction[0],
                    next[0] + 0.004 * direction[1], next[1] - 0.002 * direction[0],
                    curr[0] - 0.004 * direction[1], curr[1] + 0.002 * direction[0],
                    next[0] + 0.004 * direction[1], next[1] - 0.002 * direction[0],
                    next[0] - 0.004 * direction[1], next[1] + 0.002 * direction[0],
                );
                lineColors.push(
                    ...color, ...color,
                    ...color, ...color,
                    ...color, ...color,
                );
            }
        }

        this.sizes.line = linePositions.length / 2;
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffers.linePosition);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(linePositions), gl.STATIC_DRAW);

        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffers.lineColor);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(lineColors), gl.STATIC_DRAW);
    }

    translateView(x, y) {
        const view = mat2d.create();
        mat2d.fromTranslation(view, vec2.fromValues(2 * x, -2 * y));
        mat2d.multiply(this.matrices.view, view, this.matrices.view);
        this.calculateModelView();
    }

    scaleView(scale, x, y) {
        const translation = vec2.fromValues(2 * (x - this.canvas.width / 2), -2 * (y - this.canvas.height / 2));
        const view = mat2d.create();
        mat2d.translate(view, view, translation);
        mat2d.scale(view, view, vec2.fromValues(scale, scale));
        mat2d.translate(view, view, vec2.negate(translation, translation));
        mat2d.multiply(this.matrices.view, view, this.matrices.view);
        this.calculateModelView();
    }

    calculateModelView() {
        const modelView2d = mat2d.create();
        mat2d.scale(modelView2d, modelView2d, vec2.fromValues(1.0 / this.canvas.width, 1.0 / this.canvas.height));
        mat2d.multiply(modelView2d, modelView2d, this.matrices.view);
        mat2d.multiply(modelView2d, modelView2d, this.matrices.model);

        mat4.identity(this.matrices.modelView);
        this.matrices.modelView[0] = modelView2d[0];
        this.matrices.modelView[1] = modelView2d[1];
        this.matrices.modelView[4] = modelView2d[2];
        this.matrices.modelView[5] = modelView2d[3];
        this.matrices.modelView[12] = modelView2d[4];
        this.matrices.modelView[13] = modelView2d[5];
    }
}

class Controller {
    async setUp(gl) {
        this.gl = gl;
        this.initializeCanvas();

        this.programInfo = {
            line: new LineProgramInfo(),
            station: new StationProgramInfo(),
        };
        this.shaderData = new ShaderData();

        await Promise.all([
            this.programInfo.line.setUp(this.gl, sources.line),
            this.programInfo.station.setUp(this.gl, sources.station),
            this.shaderData.setUp(this.gl),
        ]);
        this.drawLoop();
        this.addControlListeners();
    }

    initializeCanvas() {
        this.resizeCanvas();
        addEventListener("resize", () => this.resizeCanvas());
    }

    addControlListeners() {
        addEventListener("resize", () => this.shaderData.calculateModelView());
        addEventListener("mousemove", event => {
            if (event.buttons) {
                this.shaderData.translateView(event.movementX, event.movementY);
            }
        });
        addEventListener("wheel", event => {
            if (event.deltaY < 0) {
                this.shaderData.scaleView(11 / 10, event.clientX, event.clientY);
            } else {
                this.shaderData.scaleView(10 / 11, event.clientX, event.clientY);
            }
        });
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

    drawLoop() {
        this.draw();
        requestAnimationFrame(() => this.drawLoop());
    }

    draw() {
        this.gl.disable(this.gl.DEPTH_TEST);
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFuncSeparate(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA, this.gl.ZERO, this.gl.ONE);

        this.clear();
        this.programInfo.line.run(this.shaderData);
        this.programInfo.station.run(this.shaderData);
    }
}

const sources = {
    line: {
        vertex: loadSource("shader/line.vert.glsl"),
        fragment: loadSource("shader/line.frag.glsl"),
    },
    station: {
        vertex: loadSource("shader/station.vert.glsl"),
        fragment: loadSource("shader/station.frag.glsl"),
    },
    data: loadSource("../data/vbb.json"),
};

const controller = new Controller();

addEventListener("load", () => {
    const canvas = document.querySelector("canvas");
    const gl = canvas.getContext("webgl", {alpha: false});
    controller.setUp(gl).catch(error => console.error(error));
});
