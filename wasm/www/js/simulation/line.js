import {SimulationRenderer} from "./renderer.js";

export class LineRenderer extends SimulationRenderer {
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
