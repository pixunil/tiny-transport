import {View} from "../../wasm/gtfs_sim_wasm.js";
import {LineRenderer} from "./line.js";
import {TrainRenderer} from "./train.js";
import {StationRenderer} from "./station.js";

export class SimulationCanvas {
    constructor(canvas) {
        this.canvas = canvas;
        this.gl = this.canvas.getContext("webgl", {alpha: false});
        this.resizeCanvasIfNecessary();
        this.clear();
        this.addControlListeners();

        this.view = new View(0.08, this.canvas.width, this.canvas.height);
        this.view.viewProjection = this.view.calculateViewProjection();

        this.renderer = {
            line: new LineRenderer(this.view),
            train: new TrainRenderer(this.view),
            station: new StationRenderer(this.view),
        };
    }

    async setUp(assets) {
        await Promise.all([
            this.renderer.line.setUp(this.gl, assets.line),
            this.renderer.train.setUp(this.gl, assets.train),
            this.renderer.station.setUp(this.gl, assets.station),
        ]);
    }

    async setUpWithModel(model) {
        this.model = model;
        await Promise.all([
            this.renderer.line.fillBuffers(this.model),
            this.renderer.station.fillBuffers(this.model),
            this.renderer.train.generateTextures(this.model),
        ]);
    }

    update() {
        this.renderer.train.fillBuffers(this.model);
    }

    resizeCanvasIfNecessary() {
        if (this.canvas.width !== this.canvas.clientWidth || this.canvas.height !== this.canvas.clientHeight) {
            this.canvas.width = this.canvas.clientWidth;
            this.canvas.height = this.canvas.clientHeight;
            this.gl.viewport(0, 0, this.gl.drawingBufferWidth, this.gl.drawingBufferHeight);

            if (this.view) {
                this.view.resize(this.canvas.width, this.canvas.height);
                this.view.viewProjection = this.view.calculateViewProjection();
            }
        }
    }

    addControlListeners() {
        this.canvas.addEventListener("mousemove", event => {
            this.updateTooltip(event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
            if (event.buttons) {
                this.view.scroll(event.movementX, event.movementY);
                this.view.viewProjection = this.view.calculateViewProjection();
            }
        });
        this.canvas.addEventListener("wheel", event => {
            const scaling = event.deltaY < 0 ? 11 / 10 : 10 / 11;
            this.view.zoom(scaling, event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
            this.view.viewProjection = this.view.calculateViewProjection();
        });
    }

    updateTooltip(x, y) {
        if (this.model) {
            const name = this.model.findStation(this.view, x, y);
            this.canvas.title = name ? name : "";
        }
    }

    clear() {
        this.gl.clearColor(0.9, 0.95, 0.95, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }

    draw() {
        this.resizeCanvasIfNecessary();

        this.gl.disable(this.gl.DEPTH_TEST);
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFuncSeparate(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA, this.gl.ZERO, this.gl.ONE);

        this.clear();
        this.renderer.line.run();
        this.renderer.train.run();
        this.renderer.station.run();
    }
}
