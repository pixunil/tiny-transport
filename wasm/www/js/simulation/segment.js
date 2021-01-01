import {SimulationRenderer} from "./renderer.js";

export class SegmentRenderer extends SimulationRenderer {
    initializeBuffers() {
        super.initializeBuffers();
        this.createBuffer("position", this.gl.FLOAT, 2);
    }

    fillBuffers(model) {
        this.count = model.segmentCount();
        this.segmentSizes = model.segmentSizes();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.segmentVertices(), this.gl.STATIC_DRAW);
    }

    run() {
        super.run();

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);

        let offset = 0;
        for (let segment = 0; segment < this.count; segment++) {
            this.gl.uniform3fv(this.uniformLocations.color, [0.2, 0.2, 0.2]);
            const trackRunSize = this.segmentSizes[segment];
            this.gl.drawArrays(this.gl.TRIANGLE_STRIP, offset, trackRunSize);
            offset += trackRunSize;
        }
    }
}
