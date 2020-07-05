import {Renderer} from "../base/renderer.js";

export class StationRenderer extends Renderer {
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
