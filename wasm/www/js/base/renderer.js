import {ProgramInfo} from "./program-info.js";

export class Renderer {
    constructor() {
        this.programInfo = new ProgramInfo();
    }

    async setUp(gl, assets) {
        this.gl = gl;
        await Promise.all([
            this.programInfo.setUp(gl, assets),
            this.createBuffers(),
        ]);
    }

    createBuffers() {}

    get uniformLocations() {
        return this.programInfo.uniformLocations;
    }

    get attributeLocations() {
        return this.programInfo.attributeLocations;
    }
}
