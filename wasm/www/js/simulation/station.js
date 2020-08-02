import {SimulationRenderer} from "./renderer.js";

export class StationRenderer extends SimulationRenderer {
    initializeBuffers() {
        super.initializeBuffers();
        this.createBuffer("id", this.gl.UNSIGNED_SHORT, 1, this.gl.INT);
        this.createBuffer("position", this.gl.FLOAT, 2);
        this.createBuffer("type", this.gl.UNSIGNED_BYTE, 1, this.gl.INT);
    }

    fillBuffers(model) {
        this.count = model.stationCount();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.id);
        const ids = [...Array(this.count + 1).keys()].slice(1);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, new Uint16Array(ids), this.gl.STATIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.stationPositions(), this.gl.STATIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.type);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.stationTypes(), this.gl.STATIC_DRAW);
    }

    run() {
        super.run();

        this.gl.uniform1f(this.uniformLocations.scaling, this.view.scaling());
        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        this.gl.drawArrays(this.gl.POINTS, 0, this.count);
    }
}
