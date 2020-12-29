import {SimulationRenderer} from "./renderer.js";

export class LineRenderer extends SimulationRenderer {
    initializeBuffers() {
        super.initializeBuffers();
        this.createBuffer("position", this.gl.FLOAT, 2);
    }

    fillBuffers(model) {
        this.count = model.lineCount();
        this.trackRunSizes = model.lineVerticesSizes();
        this.colors = model.lineColors();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.lineVertices(), this.gl.STATIC_DRAW);
    }

    run() {
        super.run();

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        let offset = 0;
        for (let line = 0; line < this.count; line++) {
            this.gl.uniform3fv(this.uniformLocations.color, this.colors.slice(3 * line, 3 * line + 3));
            const trackRunSize = this.trackRunSizes[line];
            this.gl.drawArrays(this.gl.TRIANGLE_STRIP, offset, trackRunSize);
            offset += trackRunSize;
        }
    }
}
