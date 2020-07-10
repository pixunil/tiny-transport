import {Renderer} from "../base/renderer.js";

export class ClockRenderer extends Renderer {
    constructor() {
        super();
        this.time = 0;
    }

    initializeBuffers() {
        super.initializeBuffers();
        this.createBuffer("position", this.gl.FLOAT, 2);
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
        super.run();

        function angle(percentage) {
            return (percentage - 0.25) * -Math.PI * 2;
        }

        this.gl.uniform1f(this.programInfo.uniformLocations.size, this.gl.canvas.width / 2);
        this.gl.uniform1f(this.programInfo.uniformLocations.minuteAngle, angle(this.time / 3600));
        this.gl.uniform1f(this.programInfo.uniformLocations.hourAngle, angle(this.time / 3600 / 12));

        this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
    }
}
