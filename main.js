import {Dataset, View, default as init} from "./wasm/gtfs_sim_wasm.js";

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
            type: this.gl.createBuffer(),
        }
    }

    fillBuffers(model) {
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.stationPositions(), this.gl.STATIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.type);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.stationTypes(), this.gl.STATIC_DRAW);

        this.count = model.stationCount();
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniform1f(this.uniformLocations.scaling, this.view.scaling());
        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.type);
        this.gl.vertexAttribPointer(
            this.attributeLocations.type,
            1, this.gl.UNSIGNED_BYTE,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.type);

        this.gl.drawArrays(this.gl.POINTS, 0, this.count);
    }
}

class LineRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
        }
    }

    fillBuffers(model) {
        this.count = model.lineCount();
        this.trackRunSizes = model.lineVerticesSizes();
        this.colors = model.lineColors();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.lineVertices(), this.gl.STATIC_DRAW);
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
        for (let line = 0; line < this.count; line++) {
            this.gl.uniform3fv(this.uniformLocations.color, this.colors.slice(3 * line, 3 * line + 3));
            for (let track = 0; track < 2; track++) {
                const trackRunSize = this.trackRunSizes[2 * line + track];
                this.gl.drawArrays(this.gl.TRIANGLE_STRIP, offset, trackRunSize);
                offset += trackRunSize;
            }
        }
    }
}

class LineNamesTextureGenerator {
    constructor(names, level) {
        this.names = names;
        this.level = level;
        const scale = 2 ** level;
        this.entryWidth = 256 / scale;
        this.entryHeight = 128 / scale;
        this.fontSize = 96 / scale;
    }

    get shape() {
        const exponent = Math.ceil(Math.log2(this.names.length));
        return [2 ** Math.floor(exponent / 2), 2 ** Math.ceil(exponent / 2)];
    }

    get width() {
        return this.shape[0] * this.entryWidth;
    }

    get height() {
        return this.shape[1] * this.entryHeight;
    }

    get ratio() {
        return this.entryHeight / this.entryWidth;
    }

    generateLineNameTexture(gl) {
        let canvas = document.createElement("canvas");
        let context = canvas.getContext("2d");
        canvas.width = this.width;
        canvas.height = this.height;

        context.fillStyle = "#ffffff";
        context.font = `${this.fontSize}px sans-serif`;
        for (let i = 0; i < this.names.length; i++) {
            const name = this.names[i];
            const measure = context.measureText(name);
            const x = (i % this.shape[0]) * this.entryWidth + (this.entryWidth - measure.width) / 2;
            const y = Math.floor(i / this.shape[0]) * this.entryHeight + this.fontSize;
            context.fillText(name, x, y);
        }

        gl.texImage2D(gl.TEXTURE_2D, this.level, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, canvas);
    }
}

class TrainRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
            color: this.gl.createBuffer(),
            extent: this.gl.createBuffer(),
            lineNumber: this.gl.createBuffer(),
            side: this.gl.createBuffer(),
        }
    }

    generateTextures(model) {
        const names = model.lineNames().split("\n");
        this.lineNamesTextureGenerator = new LineNamesTextureGenerator(names, 0);

        this.texture = this.gl.createTexture();
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.lineNamesTextureGenerator.generateLineNameTexture(this.gl);
        const maxLevel = Math.log2(Math.max(this.lineNamesTextureGenerator.width, this.lineNamesTextureGenerator.height));
        for (let level = 1; level <= maxLevel; level++) {
            const lineNamesTextureGenerator = new LineNamesTextureGenerator(names, level);
            lineNamesTextureGenerator.generateLineNameTexture(this.gl);
        }

        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.LINEAR_MIPMAP_LINEAR);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
    }

    fillBuffers(model, timePassed) {
        model.update(timePassed);
        this.count = model.trainCount();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainVertices(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.color);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainColors(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.extent);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainExtents(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.lineNumber);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainLineNumbers(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.side);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainSides(), this.gl.DYNAMIC_DRAW);
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);
        this.gl.uniform2fv(this.uniformLocations.lineNamesShape, this.lineNamesTextureGenerator.shape);
        this.gl.uniform1f(this.uniformLocations.lineNamesRatio, this.lineNamesTextureGenerator.ratio);

        this.gl.activeTexture(this.gl.TEXTURE0);
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.gl.uniform1i(this.uniformLocations.lineNames, 0);

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

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.extent);
        this.gl.vertexAttribPointer(
            this.attributeLocations.extent,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.extent);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.lineNumber);
        this.gl.vertexAttribPointer(
            this.attributeLocations.lineNumber,
            1, this.gl.UNSIGNED_SHORT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.lineNumber);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.side);
        this.gl.vertexAttribPointer(
            this.attributeLocations.side,
            2, this.gl.UNSIGNED_BYTE,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.side);

        this.gl.drawArrays(this.gl.TRIANGLES, 0, 6 * this.count);
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
            this.renderer.train.generateTextures(this.model),
        ]);

        this.milliseconds = performance.now();
        this.drawLoop(this.milliseconds);
        this.addControlListeners();
    }

    async setUpView() {
        await this.wasm;
        this.view = new View(0.08, this.canvas.width, this.canvas.height);
        this.view.viewProjection = this.view.calculateViewProjection();
    }

    async setUpModel() {
        const data = new Uint8Array(await sources.data);
        this.model = Dataset.parse(data);
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
        const name = this.model.findStation(this.view, x, y);
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
