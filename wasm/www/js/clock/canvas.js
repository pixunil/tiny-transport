import {Canvas} from "../base/canvas.js";
import {ClockRenderer} from "./clock.js";

export class ClockCanvas extends Canvas {
    constructor(canvas) {
        super(canvas);
        this.renderer = new ClockRenderer();
    }

    async setUp(assets) {
        await this.renderer.setUp(this.gl, assets.clock);
    }

    resizeCanvas() {
        this.canvas.style.height = `${this.canvas.clientWidth}px`;
        super.resizeCanvas();
    }

    update(timePassed) {
        this.renderer.update(timePassed);
    }

    draw() {
        super.draw();
        this.renderer.run();
    }
}
