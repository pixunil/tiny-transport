import {Renderer} from "../base/renderer.js";

export class ClockRenderer extends Renderer {
    constructor() {
        super();
        this.time = 0;
    }

    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
        };
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        const bufferData = new Float32Array([
            -1, +1,
            -1, -1,
            +1, +1,
            +1, -1,
        ]);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, bufferData, this.gl.STATIC_DRAW);
    }

    update(timePassed) {
        this.time += timePassed;
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        function angle(percentage) {
            return (percentage - 0.25) * -Math.PI * 2;
        }

        this.gl.uniform1f(this.programInfo.uniformLocations.size, this.gl.canvas.width / 2);
        this.gl.uniform1f(this.programInfo.uniformLocations.minuteAngle, angle(this.time / 3600));
        this.gl.uniform1f(this.programInfo.uniformLocations.hourAngle, angle(this.time / 3600 / 12));

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.programInfo.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.programInfo.attributeLocations.position);

        this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
    }
}
