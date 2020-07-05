export class Canvas {
    constructor(canvas) {
        this.canvas = canvas;
        this.gl = this.canvas.getContext("webgl2", {alpha: false});
        this.resizeCanvasIfNecessary();
        this.clear();
    }

    resizeCanvasIfNecessary() {
        if (this.canvas.width !== this.canvas.clientWidth || this.canvas.height !== this.canvas.clientHeight) {
            this.resizeCanvas();
        }
    }

    resizeCanvas() {
        this.canvas.width = this.canvas.clientWidth;
        this.canvas.height = this.canvas.clientHeight;
        this.gl.viewport(0, 0, this.gl.drawingBufferWidth, this.gl.drawingBufferHeight);
    }

    clear() {
        this.gl.clearColor(1.0, 1.0, 1.0, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }

    draw() {
        this.resizeCanvasIfNecessary();

        this.gl.disable(this.gl.DEPTH_TEST);
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFuncSeparate(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA, this.gl.ZERO, this.gl.ONE);

        this.clear();
    }
}
