import {Map, View, default as init} from "./wasm/gtfs_sim_wasm.js";

function loadSource(url) {
    return new Promise((resolve, reject) => {
        let request = new XMLHttpRequest;
        request.open("get", url);
        if (url.endsWith(".bin")) {
            request.responseType = "arraybuffer";
        }
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

class Renderer {
    constructor(view) {
        this.view = view;
        this.programInfo = new ProgramInfo();
    }

    async setUp(gl, sources) {
        this.gl = gl;
        await Promise.all([
            this.programInfo.setUp(gl, sources),
            this.createBuffers(),
        ]);
    }

    get uniformLocations() {
        return this.programInfo.uniformLocations;
    }

    get attributeLocations() {
        return this.programInfo.attributeLocations;
    }
}

class StationRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
        }
    }

    fillBuffers(model) {
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.station_positions(), this.gl.STATIC_DRAW);

        this.size = model.station_size();
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniform1f(this.uniformLocations.size, 90.0 * this.view.scaling());
        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.drawArrays(this.gl.POINTS, 0, this.size);
    }
}

class LineRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
        }
    }

    fillBuffers(model) {
        this.lineSizes = model.line_sizes();
        this.trackRunSizes = model.track_run_sizes();
        this.colors = model.line_colors();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.line_vertices(), this.gl.STATIC_DRAW);
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        let offset = 0;
        this.lineSizes.reduce((start, count, index) => {
            this.gl.uniform3fv(this.uniformLocations.color, this.colors.slice(3 * index, 3 * index + 3));

            const stop = start + count;
            for (let trackRunSize of this.trackRunSizes.slice(start, stop)) {
                this.gl.drawArrays(this.gl.TRIANGLE_STRIP, offset, trackRunSize);
                offset += trackRunSize;
            }
            return stop;
        }, 0);
    }
}

class TrainRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
            color: this.gl.createBuffer(),
        }
    }

    fillBuffers(model, timePassed) {
        model.update(timePassed);
        this.size = model.train_size();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.train_vertices(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.color);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.train_colors(), this.gl.DYNAMIC_DRAW);
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.color);
        this.gl.vertexAttribPointer(
            this.attributeLocations.color,
            3, this.gl.FLOAT,
            false, 0, 0);
            this.gl.enableVertexAttribArray(this.attributeLocations.color);

        this.gl.drawArrays(this.gl.TRIANGLES, 0, 6 * this.size);
    }
}

class Controller {
    constructor() {
        this.wasm = init("wasm/gtfs_sim_wasm_bg.wasm");
    }

    async setUp(gl) {
        this.gl = gl;
        this.canvas = gl.canvas;
        this.resizeCanvasIfNecessary();
        this.clear();

        await this.setUpView();

        this.renderer = {
            line: new LineRenderer(this.view),
            train: new TrainRenderer(this.view),
            station: new StationRenderer(this.view),
        };

        await Promise.all([
            this.setUpModel(),
            this.renderer.line.setUp(this.gl, sources.line),
            this.renderer.train.setUp(this.gl, sources.train),
            this.renderer.station.setUp(this.gl, sources.station),
        ]);
        await Promise.all([
            this.renderer.line.fillBuffers(this.model),
            this.renderer.station.fillBuffers(this.model),
        ]);

        this.milliseconds = performance.now();
        this.drawLoop(this.milliseconds);
        this.addControlListeners();
    }

    async setUpView() {
        await this.wasm;
        this.view = new View(this.canvas.width, this.canvas.height);
        this.view.viewProjection = this.view.calculateViewProjection();
    }

    async setUpModel() {
        const data = new Uint8Array(await sources.data);
        this.model = Map.parse(data);
        this.model.update(14010);
    }

    addControlListeners() {
        this.canvas.addEventListener("mousemove", event => {
            this.updateTooltip(event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
            if (event.buttons) {
                this.view.scroll(event.movementX, event.movementY);
                this.view.viewProjection = this.view.calculateViewProjection();
            }
        });
        this.canvas.addEventListener("wheel", event => {
            const scaling = event.deltaY < 0 ? 11 / 10 : 10 / 11;
            this.view.zoom(scaling, event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
            this.view.viewProjection = this.view.calculateViewProjection();
        });
    }

    updateTooltip(x, y) {
        const name = this.model.find_station(this.view, x, y);
        this.gl.canvas.title = name ? name : "";
    }

    resizeCanvasIfNecessary() {
        if (this.canvas.width !== this.canvas.clientWidth || this.canvas.height !== this.canvas.clientHeight) {
            this.canvas.width = this.canvas.clientWidth;
            this.canvas.height = this.canvas.clientHeight;
            this.gl.viewport(0, 0, this.gl.drawingBufferWidth, this.gl.drawingBufferHeight);

            if (this.view) {
                this.view.resize(this.canvas.width, this.canvas.height);
                this.view.viewProjection = this.view.calculateViewProjection();
            }
        }
    }

    clear() {
        this.gl.clearColor(0.9, 0.95, 0.95, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }

    drawLoop(milliseconds) {
        this.resizeCanvasIfNecessary();
        this.update(milliseconds);
        this.draw();
        requestAnimationFrame(milliseconds => this.drawLoop(milliseconds));
    }

    update(milliseconds) {
        const speed = parseInt(document.querySelector("input").value);
        const millisecondsPassed = milliseconds - this.milliseconds;
        this.milliseconds = milliseconds;
        const timePassed = Math.floor(millisecondsPassed * speed / 1000);
        this.renderer.train.fillBuffers(this.model, timePassed);
    }

    draw() {
        this.gl.disable(this.gl.DEPTH_TEST);
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFuncSeparate(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA, this.gl.ZERO, this.gl.ONE);

        this.clear();
        this.renderer.line.run();
        this.renderer.train.run();
        this.renderer.station.run();
    }
}

const sources = {
    line: {
        vertex: loadSource("shader/line.vert.glsl"),
        fragment: loadSource("shader/line.frag.glsl"),
    },
    train: {
        vertex: loadSource("shader/train.vert.glsl"),
        fragment: loadSource("shader/train.frag.glsl"),
    },
    station: {
        vertex: loadSource("shader/station.vert.glsl"),
        fragment: loadSource("shader/station.frag.glsl"),
    },
    data: loadSource("data.bin"),
};

const controller = new Controller();

addEventListener("load", () => {
    const canvas = document.querySelector("canvas");
    const gl = canvas.getContext("webgl", {alpha: false});
    controller.setUp(gl).catch(error => console.error(error));
});
