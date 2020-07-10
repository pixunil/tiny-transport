import {ProgramInfo} from "./program-info.js";

export class Renderer {
    constructor() {
        this.programInfo = new ProgramInfo();
    }

    async setUp(gl, assets) {
        this.gl = gl;
        await this.programInfo.setUp(gl, assets);
        this.initializeBuffers();
    }

    initializeBuffers() {
        this.buffers = {};
        this.vertexArray = this.gl.createVertexArray();
        this.gl.bindVertexArray(this.vertexArray);
    }

    createBuffer(name, type, size, kind = WebGL2RenderingContext.FLOAT) {
        const buffer = this.gl.createBuffer();
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, buffer);
        if (kind === this.gl.FLOAT) {
            this.gl.vertexAttribPointer(
                this.programInfo.attributeLocations[name], size, type,
                false, 0, 0,
            );
        } else {
            this.gl.vertexAttribIPointer(
                this.programInfo.attributeLocations[name], size, type,
                0, 0,
            );
        }
        this.gl.enableVertexAttribArray(this.programInfo.attributeLocations[name]);
        this.buffers[name] = buffer;
    }

    get uniformLocations() {
        return this.programInfo.uniformLocations;
    }

    run() {
        this.gl.useProgram(this.programInfo.program);
        this.gl.bindVertexArray(this.vertexArray);
    }
}
